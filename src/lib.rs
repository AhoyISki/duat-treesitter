//! A [tree-sitter] implementation for Duat
//!
//! `duat-treesitter` currently does two things:
//!
//! * Syntax highlighting
//! * Indentation calculation
//!
//! # Installation
//!
//! Just like other Duat plugins, this one can be installed by calling
//! `cargo add` in the config directory:
//!
//! ```bash
//! cargo add duat-treesitter@"*" --rename treesitter
//! ```
//!
//! Or, if you are using a `--git-deps` version of duat, do this:
//!
//! ```bash
//! cargo add --git https://github.com/AhoyISki/duat-treesitter --rename treesitter
//! ```
//!
//! But this is a default plugin, so you most likely won't have to do
//! that.
//!
//! [tree-sitter]: https://tree-sitter.github.io/tree-sitter
#![feature(closure_lifetime_binder)]
use std::{
    collections::HashMap,
    fs,
    ops::Range,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use duat_core::{
    file::{self, PathKind},
    form::FormId,
    hook::OnFileOpen,
    mode::Cursor,
    prelude::*,
    text::{Builder, Bytes, Change, Matcheable, Moment, Point, Tags},
};
use duat_filetype::FileType;
use streaming_iterator::StreamingIterator;
use tree_sitter::{
    InputEdit, Language, Node, Parser, Point as TSPoint, Query, QueryCapture as QueryCap,
    QueryCursor, QueryMatch, TextProvider, Tree,
};

use crate::languages::parser_is_compiled;

mod cursor;
mod languages;

/// The [tree-sitter] plugin for Duat
///
/// For now, it adds syntax highlighting and indentation, but more
/// features will be coming in the future.
///
/// These things are done through the [`TsParser`] [`Parser`], which
/// reads updates the inner syntax tree when the [`Text`] reports any
/// changes.
///
/// # NOTE
///
/// If you are looking to create a [`Parser`] which can do similar
/// things, you should look at the code for the implementation of
/// [`Parser`] for [`TsParser`], it's relatively short and with good
/// explanations for what is happening.
///
/// [tree-sitter]: https://tree-sitter.github.io/tree-sitter
pub struct TreeSitter;

impl<U: duat_core::ui::Ui> duat_core::Plugin<U> for TreeSitter {
    fn plug(self) {
        form::set_many!(
            ("variable", Form::white()),
            ("variable.builtin", Form::dark_yellow()),
            ("constant", Form::grey()),
            ("constant.builtin", Form::dark_yellow()),
            ("module", Form::blue().italic()),
            ("label", Form::green()),
            ("string", Form::green()),
            ("character", Form::dark_yellow()),
            ("boolean", Form::dark_yellow()),
            ("number", Form::dark_yellow()),
            ("type", Form::yellow().italic()),
            ("type.builtin", Form::yellow().reset()),
            ("attribute", Form::green()),
            ("property", Form::green()),
            ("function", Form::blue().reset()),
            ("constructor", Form::dark_yellow().reset()),
            ("operator", Form::cyan()),
            ("keyword", Form::magenta()),
            ("punctuation.bracket", Form::grey()),
            ("punctuation.delimiter", Form::grey()),
            ("comment", Form::grey()),
            ("comment.documentation", Form::grey().bold()),
            ("markup.strong", Form::bold()),
            ("markup.italic", Form::italic()),
            ("markup.strikethrough", Form::crossed_out()),
            ("markup.underline", Form::underlined()),
            ("markup.heading", Form::blue().bold()),
            ("markup.math", Form::yellow()),
            ("markup.quote", Form::grey().italic()),
            ("markup.link", Form::blue().underlined()),
            ("markup.raw", Form::cyan()),
            ("markup.list", Form::yellow()),
            ("markup.list.checked", Form::green()),
            ("markup.list.unchecked", Form::grey()),
            ("diff.plus", Form::red()),
            ("diff.delta", Form::blue()),
            ("diff.minus", Form::green()),
            ("node.field", "variable.member"),
        );

        hook::add_grouped::<OnFileOpen<U>, U>("TreeSitter", |pa, builder| {
            builder.add_parser(pa, TsParser::new());
        });
    }
}

/// [`Parser`] that parses [`File`]'s as [tree-sitter] syntax trees
///
/// [tree-sitter]: https://tree-sitter.github.io/tree-sitter
pub struct TsParser(Option<InnerTsParser>);

impl TsParser {
    /// Returns a new instance of a [`TsParser`]
    pub fn new() -> Self {
        Self(None)
    }

    /// The root [`Node`] of the syntax tree
    pub fn root(&self) -> Option<Node<'_>> {
        self.0.as_ref().map(|inner| inner.tree.root_node())
    }

    /// Gets the requested indentation level on a given [`Point`]
    ///
    /// Will be [`None`] if the [`filetype`] hasn't been set yet or if
    /// there is no indentation query for this language.
    ///
    /// [`filetype`]: FileType::filetype
    pub fn indent_on(&self, p: Point, bytes: &Bytes, cfg: PrintCfg) -> Option<usize> {
        self.0
            .as_ref()
            .and_then(|inner| inner.indent_on(p, bytes, cfg))
    }
}

impl<U: Ui> file::Parser<U> for TsParser {
    fn parse(&mut self, _: &mut Pass, snap: FileSnapshot, ranges: Option<&mut Ranges>) {
        file::Parser::<U>::parse_remote(self, snap, ranges);
    }

    fn update_range(&mut self, mut parts: FileParts<U>, within: Option<Range<Point>>) {
        if let Some(inner) = self.0.as_mut()
            && let Some(within) = within
        {
            let range = within.start.byte()..within.end.byte();
            inner.highlight_and_inject(parts.bytes, &mut parts.tags, range);
        }
    }

    fn parse_remote(&mut self, snap: FileSnapshot, ranges: Option<&mut Ranges>) {
        let Some(inner) = self.0.as_mut() else {
            return;
        };

        fn merge_tree_changed_ranges(parser: &InnerTsParser, list: &mut Ranges) {
            if let Some(old_tree) = parser.old_tree.as_ref() {
                for range in parser.tree.changed_ranges(old_tree) {
                    list.add(range.start_byte..range.end_byte);
                }

                for st in parser.sub_trees.iter() {
                    merge_tree_changed_ranges(st, list)
                }
            }
        }

        inner.apply_changes(snap.bytes, snap.moment);

        if let Some(ranges) = ranges {
            // This initial check might find larger, somewhat self contained nodes
            // that have changed, e.g. an identifier that is now recognized as a
            // function, things of that sort.
            merge_tree_changed_ranges(inner, ranges);

            // However, `changed_ranges` doesn't catch everything, so another
            // check is done. At a minimum, at least the lines where the changes
            // took place should be updated.
            for change in snap.moment.changes() {
                let start = change.start();
                let added = change.added_end();
                let start = snap.bytes.point_at_line(start.line());
                let end = snap
                    .bytes
                    .point_at_line((added.line() + 1).min(snap.bytes.len().line()));
                ranges.add(start.byte()..end.byte());
            }
        }
    }

    fn make_remote(&self) -> bool
    where
        Self: Send,
    {
        self.0
            .as_ref()
            .is_some_and(|inner| inner.tree.root_node().is_error())
    }
}

impl<U: Ui> ParserCfg<U> for TsParser {
    type Parser = Self;

    fn init(self, file: &File<U>) -> Result<ParserBox<U>, Text> {
        let path = file.path_kind();
        let filetype = if let PathKind::SetExists(path) | PathKind::SetAbsent(path) = &path
            && let Some(filetype) = path.filetype()
        {
            filetype
        } else {
            context::debug!(
                "No filetype set for {}, will try again once one is set",
                path.name_txt()
            );
            return Ok(ParserBox::new_local(file, self));
        };

        const MAX_LEN_FOR_LOCAL: usize = 100_000;
        let offset = TSPoint::default();
        let len = file.bytes().len();

        if parser_is_compiled(filetype)? && file.bytes().len().byte() <= MAX_LEN_FOR_LOCAL {
            let lang_parts = lang_parts(filetype)?;
            let form_parts = forms_from_lang_parts(&lang_parts);

            let inner =
                InnerTsParser::new(file.bytes(), 0..len.byte(), offset, lang_parts, form_parts);

            Ok(ParserBox::new_send(file, Self(Some(inner))))
        } else {
            Ok(ParserBox::new_remote(file, move |bytes| {
                let lang_parts = lang_parts(filetype)?;
                let form_parts = forms_from_lang_parts(&lang_parts);

                Ok(Self(Some(InnerTsParser::new(
                    bytes,
                    0..len.byte(),
                    offset,
                    lang_parts,
                    form_parts,
                ))))
            }))
        }
    }
}

impl Default for TsParser {
    fn default() -> Self {
        Self::new()
    }
}

struct InnerTsParser {
    parser: Parser,
    range: Range<usize>,
    offset: TSPoint,
    lang_parts: LangParts<'static>,
    forms: &'static [(FormId, u8)],
    tree: Tree,
    old_tree: Option<Tree>,
    sub_trees: Vec<InnerTsParser>,
    tagger: Tagger,
}

impl InnerTsParser {
    fn new(
        bytes: &Bytes,
        range: Range<usize>,
        offset: TSPoint,
        lang_parts: LangParts<'static>,
        forms: &'static [(FormId, u8)],
    ) -> InnerTsParser {
        let (.., lang, _) = &lang_parts;

        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser
            .parse_with_options(&mut buf_parse(bytes, range.clone()), None, None)
            .unwrap();

        InnerTsParser {
            parser,
            range,
            offset,
            lang_parts,
            forms,
            tree,
            old_tree: None,
            sub_trees: Vec::new(),
            tagger: ts_tagger(),
        }
    }

    fn highlight_and_inject(&mut self, bytes: &Bytes, tags: &mut Tags, range: Range<usize>) {
        if range.start >= self.range.end || range.end <= self.range.start {
            return;
        }

        let (.., Queries { highlights, injections, .. }) = &self.lang_parts;
        let buf = TsBuf(bytes);

        tags.remove(self.tagger, range.clone());
        // Include a little bit of overhang, in order to deal with some loose
        // ends, mostly related to comments.
        // There should be no tag duplication, since Duat does not allow that.
        let start = range.start.saturating_sub(1).max(self.range.start);
        let end = (range.end + 1).min(bytes.len().byte()).min(self.range.end);

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(range.clone());
        let root = self
            .tree
            .root_node_with_offset(self.range.start, self.offset);

        let sub_trees_to_add = {
            type SubTreeToAdd = (Range<usize>, TSPoint, LangParts<'static>);
            let mut to_add: Vec<SubTreeToAdd> = Vec::new();

            let mut inj_captures = cursor.captures(injections, root, buf);
            let cn = injections.capture_names();

            let is_content = |cap: &&QueryCap| cn[cap.index as usize] == "injection.content";
            let is_language = |cap: &&QueryCap| cn[cap.index as usize] == "injection.language";

            while let Some((qm, _)) = inj_captures.next() {
                let Some(cap) = qm.captures.iter().find(is_content) else {
                    continue;
                };

                let props = injections.property_settings(qm.pattern_index);
                let lang = props
                    .iter()
                    .find_map(|p| {
                        (p.key.as_ref() == "injection.language")
                            .then_some(p.value.as_ref().unwrap().to_string())
                    })
                    .or_else(|| {
                        let cap = qm.captures.iter().find(is_language)?;
                        Some(
                            bytes
                                .buffers(cap.node.byte_range())
                                .try_to_string()
                                .unwrap(),
                        )
                    });

                let range = cap.node.byte_range();
                let key_fn = |(range, ..): &(Range<usize>, _, _)| range.start;
                if let Some(lang) = lang
                    && let Ok(mut lang_parts) = lang_parts(&lang)
                    && let Err(i) = to_add.binary_search_by_key(&range.start, key_fn)
                {
                    let start = cap.node.start_position();

                    // You may want to set a new injections query, only for this capture.
                    if let Some(prop) = props.iter().find(|p| p.key.as_ref() == "injection.query")
                        && let Some(value) = prop.value.as_ref()
                    {
                        match query_from_path(&lang, value, lang_parts.1) {
                            Ok(injections) => lang_parts.2.injections = injections,
                            Err(err) => context::error!("{err}"),
                        }
                    }

                    to_add.insert(i, (range, start, lang_parts));
                }
            }

            to_add
        };

        let buf = TsBuf(bytes);
        cursor.set_byte_range(start..end);
        let mut hi_captures = cursor.captures(highlights, root, buf);
        while let Some((qm, _)) = hi_captures.next() {
            for cap in qm.captures.iter() {
                let range = cap.node.range();
                let (start, end) = (range.start_byte, range.end_byte);
                let (form, priority) = self.forms[cap.index as usize];
                if start != end {
                    tags.insert(self.tagger, start..end, form.to_tag(priority));
                }
            }
        }

        // If a tree was not in sub_trees_to_add, but is part of the affected
        // range, that means it was removed.
        self.sub_trees.retain_mut(|st| {
            if let Some((.., lp)) = sub_trees_to_add.iter().find(|(lhs, ..)| *lhs == st.range) {
                let hi_range = (st.range.start + 1).min(st.range.end)..st.range.end;
                if lp.0 == st.lang_parts.0 {
                    st.highlight_and_inject(bytes, tags, hi_range);
                    true
                } else {
                    if !(st.range.start >= start && st.range.end <= end) {
                        tags.remove(self.tagger, hi_range);
                    }
                    false
                }
            // If the sub tree is not in sub_trees_to_add, but its
            // range was parsed, it has been declared as deleted.
            // Unless it is a non-empty duat-text sub tree, in which
            // case this rule is ignored
            } else if st.range.start >= start && st.range.end <= end {
                false
            } else {
                st.highlight_and_inject(bytes, tags, range.clone());
                true
            }
        });

        // In the end, we add the sub trees that weren't already in there.
        // This should automatically handle all of the sub trees's sub trees.
        for (st_range, offset, lang_parts) in sub_trees_to_add {
            let key_fn = |st: &InnerTsParser| st.range.start;

            let Err(i) = self.sub_trees.binary_search_by_key(&st_range.start, key_fn) else {
                continue;
            };

            let form_parts = forms_from_lang_parts(&lang_parts);
            let hi_range = (st_range.start + 1).min(st_range.end)..st_range.end;
            let mut st = InnerTsParser::new(bytes, st_range, offset, lang_parts, form_parts);
            st.highlight_and_inject(bytes, tags, hi_range);
            self.sub_trees.insert(i, st)
        }
    }

    fn apply_sub_tree_change(&mut self, change: Change<&str>, bytes: &Bytes) {
        let start = change.start();
        let added = change.added_end();
        let taken = change.taken_end();

        // By this point, if this tree were to be clipped by the change, it
        // would have been removed already.
        if start.byte() < self.range.start && taken.byte() <= self.range.start {
            let ts_start = ts_point(start, bytes);
            let ts_taken_end = ts_point_from(taken, (ts_start.column, start), change.taken_str());
            let ts_added_end = ts_point_from(added, (ts_start.column, start), change.added_str());

            self.range.start = (self.range.start as i32 + change.shift()[0]) as usize;
            self.range.end = (self.range.end as i32 + change.shift()[0]) as usize;
            self.offset = deoffset(self.offset, ts_taken_end);
            self.offset = reoffset(self.offset, ts_added_end);
        } else if taken.byte() <= self.range.end {
            let edit = input_edit(change, bytes, self.offset, self.range.start);
            self.tree.edit(&edit);
            self.range.end = (self.range.end as i32 + change.shift()[0]) as usize;
            self.old_tree = None;
        } else {
            // If this sub tree wasn't affected, neither will any of its children.
            return;
        }

        self.sub_trees.retain_mut(|st| {
            if change_clips(change, st.range.clone()) {
                false
            } else {
                st.apply_sub_tree_change(change, bytes);
                true
            }
        });
    }

    fn shift_tree(&mut self, shift: [i32; 3]) {
        self.range.start = (self.range.start as i32 + shift[0]) as usize;
        self.range.end = (self.range.end as i32 + shift[0]) as usize;
        self.offset.row = (self.offset.row as i32 + shift[2]) as usize;
        for st in self.sub_trees.iter_mut() {
            st.shift_tree(shift);
        }
    }

    fn reparse_sub_trees(&mut self, bytes: &Bytes) {
        for st in self.sub_trees.iter_mut() {
            st.reparse_sub_trees(bytes);
            if st.old_tree.is_none() {
                let mut parse_fn = buf_parse(bytes, st.range.clone());
                let tree = st
                    .parser
                    .parse_with_options(&mut parse_fn, Some(&st.tree), None)
                    .unwrap();

                st.old_tree = Some(std::mem::replace(&mut st.tree, tree));
            }
        }
    }

    fn apply_changes(&mut self, bytes: &Bytes, moment: Moment) {
        fn changes_b_shift(changes: &[Change<&str>]) -> i32 {
            changes
                .iter()
                .fold(0, |b_sh, change| b_sh + change.shift()[0])
        }

        let mut sub_trees_to_remove = Vec::new();
        let mut sub_trees = self.sub_trees.iter_mut().enumerate().peekable();
        let mut last_changes: Vec<Change<&str>> = Vec::new();
        let (mut sh_from, mut shift) = (0, [0; 3]);

        for change in moment.changes() {
            let start = change.start();

            let input_edit = input_edit(change, bytes, self.offset, self.range.start);
            self.tree.edit(&input_edit);

            self.range.end = (self.range.end as i32 + change.shift()[0]) as usize;

            // First, deal with all sub trees before the Change.
            while let Some((i, st)) = sub_trees.next_if(|(i, st)| {
                let end = if *i >= sh_from {
                    (st.range.end as i32 + shift[0] + changes_b_shift(&last_changes)) as usize
                } else {
                    st.range.end
                };
                end < start.byte()
            }) {
                if i == sh_from {
                    st.shift_tree(shift);
                    for change in last_changes.iter() {
                        st.apply_sub_tree_change(*change, bytes);
                    }
                }

                sh_from = i + 1;
            }

            // Then, get rid of consecutively clipped sub trees.
            while let Some((i, _)) = sub_trees.next_if(|(i, st)| {
                let range = if *i == sh_from {
                    let shift = shift[0] + changes_b_shift(&last_changes);
                    let start = (st.range.start as i32 + shift) as usize;
                    let end = (st.range.end as i32 + shift) as usize;
                    start..end
                } else {
                    st.range.clone()
                };
                change_clips(change, range)
            }) {
                sub_trees_to_remove.push(i);
                sh_from = i + 1;
            }

            // Now, this sub tree should either contain the change or be ahead of
            // it.
            if let Some((i, st)) = sub_trees.peek_mut() {
                if *i == sh_from {
                    st.shift_tree(shift);
                    for change in last_changes.iter() {
                        st.apply_sub_tree_change(*change, bytes);
                    }
                }

                st.apply_sub_tree_change(change, bytes);
                sh_from = *i + 1;

                if let Some(last) = last_changes.last()
                    && (last.taken_end().line() == start.line()
                        || last.taken_end().byte() > st.range.start)
                {
                    last_changes.push(change);
                } else {
                    shift = last_changes
                        .drain(..)
                        .fold(shift, |sh, change| add_shifts(sh, change.shift()));
                    last_changes = vec![change];
                };
            }
        }

        for (i, st) in sub_trees {
            if i >= sh_from {
                st.shift_tree(shift);
                for change in last_changes.iter() {
                    st.apply_sub_tree_change(*change, bytes);
                }
            }
        }

        let mut parse_fn = buf_parse(bytes, self.range.clone());
        let tree = self
            .parser
            .parse_with_options(&mut parse_fn, Some(&self.tree), None)
            .unwrap();

        self.old_tree = Some(std::mem::replace(&mut self.tree, tree));
        drop(parse_fn);

        for i in sub_trees_to_remove.into_iter().rev() {
            self.sub_trees.remove(i);
        }

        self.reparse_sub_trees(bytes);
    }

    fn indent_on(&self, p: Point, bytes: &Bytes, cfg: PrintCfg) -> Option<usize> {
        fn smallest_around(
            ts: &InnerTsParser,
            b: usize,
        ) -> (Node<'_>, &Query, [usize; 2], impl Fn(usize) -> usize) {
            if let Some(sub) = ts
                .sub_trees
                .iter()
                .find(|sub| sub.range.contains(&b) && sub.lang_parts.0 != "comment")
            {
                return smallest_around(sub, b);
            }

            let offset = [ts.range.start, ts.offset.row];
            let col_off = |line: usize| ts.offset.column * (line == ts.offset.row) as usize;

            (
                ts.tree.root_node(),
                ts.lang_parts.2.indents,
                offset,
                col_off,
            )
        }

        let [start, _] = bytes.points_of_line(p.line());
        let (root, indents, [byte_off, line_off], col_off) = smallest_around(self, start.byte());

        // The query could be empty.
        if indents.pattern_count() == 0 {
            return None;
        }

        // TODO: Don't reparse python, apparently.

        type Captures<'a> = HashMap<&'a str, HashMap<usize, HashMap<&'a str, Option<&'a str>>>>;
        let mut caps = HashMap::new();
        let q = {
            let mut cursor = QueryCursor::new();
            let buf = TsBuf(bytes);
            cursor
                .matches(indents, root, buf)
                .for_each(|qm: &QueryMatch| {
                    for cap in qm.captures.iter() {
                        let Some(cap_end) =
                            indents.capture_names()[cap.index as usize].strip_prefix("indent.")
                        else {
                            continue;
                        };

                        let nodes = if let Some(nodes) = caps.get_mut(cap_end) {
                            nodes
                        } else {
                            caps.insert(cap_end, HashMap::new());
                            caps.get_mut(cap_end).unwrap()
                        };
                        let props = indents.property_settings(qm.pattern_index).iter();
                        nodes.insert(
                            cap.node.id(),
                            props
                                .map(|p| {
                                    let key = p.key.strip_prefix("indent.").unwrap();
                                    (key, p.value.as_deref())
                                })
                                .collect(),
                        );
                    }
                });
            |caps: &Captures, node: Node, queries: &[&str]| {
                caps.get(queries[0])
                    .and_then(|nodes| nodes.get(&node.id()))
                    .is_some_and(|props| {
                        let key = queries.get(1);
                        key.is_none_or(|key| props.iter().any(|(k, _)| k == key))
                    })
            }
        };

        // The first non indent character of this line.
        let indented_start = bytes
            .chars_fwd(start..)
            .unwrap()
            .take_while(|(p, _)| p.line() == start.line())
            .find_map(|(p, c)| (!c.is_whitespace()).then_some(p));

        let mut opt_node = if let Some(indented_start) = indented_start {
            Some(descendant_in(root, indented_start.byte(), byte_off))
        // If the line is empty, look behind for another.
        } else {
            // Find last previous empty line.
            let mut lines = bytes.lines(..start).rev();
            let Some((prev_l, line)) = lines
                .find(|(_, line)| !(line.reg_matches(r"^\s*$", ..).unwrap()))
                .filter(|(l, _)| *l >= line_off)
            else {
                // If there is no previous non empty line, align to 0.
                return Some(0);
            };
            let trail = line.chars().rev().take_while(|c| c.is_whitespace()).count();

            let [prev_start, prev_end] = bytes.points_of_line(prev_l);
            let mut node = descendant_in(root, prev_end.byte() - (trail + 1), byte_off);
            if node.kind().contains("comment") {
                // Unless the whole line is a comment, try to find the last node
                // before the comment.
                // This technically fails if there are multiple block comments.
                let first_node = descendant_in(root, prev_start.byte(), byte_off);
                if first_node.id() != node.id() {
                    node = descendant_in(root, node.start_byte() - 1, 0)
                }
            }

            Some(if q(&caps, node, &["end"]) {
                descendant_in(root, start.byte(), byte_off)
            } else {
                node
            })
        };

        if q(&caps, opt_node.unwrap(), &["zero"]) {
            return Some(0);
        }

        let tab = cfg.tab_stops.size() as i32;
        let mut indent = if root.start_byte() + byte_off != 0 {
            bytes.indent(bytes.point_at_byte(root.start_byte() + byte_off), cfg) as i32
        } else {
            0
        };

        let mut processed_lines = Vec::new();
        while let Some(node) = opt_node {
            let s_line = node.start_position().row + line_off;
            let e_line = node.end_position().row + line_off;

            // If a node is not an indent and is marked as auto or ignore, act
            // accordingly.
            if !q(&caps, node, &["begin"]) && s_line < p.line() && p.line() <= e_line {
                if !q(&caps, node, &["align"]) && q(&caps, node, &["auto"]) {
                    return None;
                } else if q(&caps, node, &["ignore"]) {
                    return Some(0);
                }
            }

            let should_process = !processed_lines.contains(&s_line);

            let mut is_processed = false;

            if should_process
                && ((s_line == p.line() && q(&caps, node, &["branch"]))
                    || (s_line != p.line() && q(&caps, node, &["dedent"])))
            {
                indent -= tab;
                is_processed = true;
            }

            let is_in_err = should_process && node.parent().is_some_and(|p| p.is_error());
            // Indent only if the node spans more than one line, or under other
            // special circumstances.
            if should_process
                && q(&caps, node, &["begin"])
                && (s_line != e_line || is_in_err || q(&caps, node, &["begin", "immediate"]))
                && (s_line != p.line() || q(&caps, node, &["begin", "start_at_same_line"]))
            {
                is_processed = true;
                indent += tab;
            }

            if is_in_err && !q(&caps, node, &["align"]) {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if q(&caps, child, &["align"]) {
                        let props = caps["align"][&child.id()].clone();
                        caps.get_mut("align").unwrap().insert(node.id(), props);
                    }
                }
            }

            let fd = for<'a, 'b> |node: Node<'a>, delim: &'b str| -> (Option<Node<'a>>, bool) {
                let mut c = node.walk();
                let child = node.children(&mut c).find(|child| child.kind() == delim);
                let ret = child.map(|child| {
                    let [_, end] = bytes.points_of_line(child.start_position().row + line_off);
                    let range = child.range().start_byte + byte_off..end.byte();

                    let is_last_in_line = if let Some(line) = bytes.get_contiguous(range.clone()) {
                        line.split_whitespace().any(|w| w != delim)
                    } else {
                        let line = bytes.buffers(range).try_to_string().unwrap();
                        line.split_whitespace().any(|w| w != delim)
                    };

                    (child, is_last_in_line)
                });
                let (child, is_last_in_line) = ret.unzip();
                (child, is_last_in_line.unwrap_or(false))
            };

            if should_process
                && q(&caps, node, &["align"])
                && (s_line != e_line || is_in_err)
                && s_line != p.line()
            {
                let props = &caps["align"][&node.id()];
                let (o_delim_node, o_is_last_in_line) = props
                    .get(&"open_delimiter")
                    .and_then(|delim| delim.map(|d| fd(node, d)))
                    .unwrap_or((Some(node), false));
                let (c_delim_node, c_is_last_in_line) = props
                    .get(&"close_delimiter")
                    .and_then(|delim| delim.map(|d| fd(node, d)))
                    .unwrap_or((Some(node), false));

                if let Some(o_delim_node) = o_delim_node {
                    let o_s_line = o_delim_node.start_position().row + line_off;
                    let o_s_col = o_delim_node.start_position().column + col_off(o_s_line);
                    let c_s_line = c_delim_node.map(|n| n.start_position().row + line_off);

                    // If the previous line was marked with an open_delimiter, treat it
                    // like an indent.
                    let indent_is_absolute = if o_is_last_in_line && should_process {
                        indent += tab;
                        // If the aligned node ended before the current line, its @align
                        // shouldn't affect it.
                        if c_is_last_in_line && c_s_line.is_some_and(|l| l < p.line()) {
                            indent = (indent - tab).max(0);
                        }
                        false
                    // Aligned indent
                    } else if c_is_last_in_line
                        && let Some(c_s_line) = c_s_line
                        // If the aligned node ended before the current line, its @align
                        // shouldn't affect it.
                        && (o_s_line != c_s_line && c_s_line < p.line())
                    {
                        indent = (indent - tab).max(0);
                        false
                    } else {
                        let inc = props.get("increment").cloned().flatten();
                        indent = o_s_col as i32 + inc.map(str::parse::<i32>).unwrap().unwrap();
                        true
                    };

                    // If this is the last line of the @align, then some additional
                    // indentation may be needed to avoid clashes. This is the case in
                    // some function parameters, for example.
                    let avoid_last_matching_next = c_s_line
                        .is_some_and(|c_s_line| c_s_line != o_s_line && c_s_line == p.line())
                        && props.contains_key("avoid_last_matching_next");
                    if avoid_last_matching_next {
                        indent += tab;
                    }
                    is_processed = true;
                    if indent_is_absolute {
                        return Some(indent as usize);
                    }
                }
            }

            if should_process && is_processed {
                processed_lines.push(s_line);
            }
            opt_node = node.parent();
        }

        // indent < 0 means "keep level of indentation"
        (indent >= 0).then_some(indent as usize)
    }
}

impl std::fmt::Debug for InnerTsParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.range.start;
        f.debug_struct("TsParser")
            .field("range", &self.range)
            .field("offset", &self.offset)
            .field("tree", &self.tree.root_node_with_offset(start, self.offset))
            .field("old_tree", &self.old_tree)
            .field("sub_trees", &self.sub_trees)
            .finish_non_exhaustive()
    }
}

#[track_caller]
fn descendant_in(node: Node, byte: usize, offset: usize) -> Node {
    node.descendant_for_byte_range(byte - offset, byte + 1 - offset)
        .unwrap()
}

fn buf_parse<'a>(bytes: &'a Bytes, range: Range<usize>) -> impl FnMut(usize, TSPoint) -> &'a [u8] {
    let [s0, s1] = bytes.buffers(range).to_array();
    |byte, _point| {
        if byte < s0.len() {
            &s0[byte..]
        } else {
            &s1[byte - s0.len()..]
        }
    }
}

fn ts_point(point: Point, buffer: &Bytes) -> TSPoint {
    let strs = buffer.strs(..point.byte()).unwrap();
    let iter = strs.into_iter().flat_map(str::chars).rev();
    let col = iter.take_while(|&b| b != '\n').count();

    TSPoint::new(point.line(), col)
}

fn ts_point_from(to: Point, (col, from): (usize, Point), str: &str) -> TSPoint {
    let col = if to.line() == from.line() {
        col + str.chars().count()
    } else {
        str.chars().rev().take_while(|&b| b != '\n').count()
    };

    TSPoint::new(to.line(), col)
}

fn forms_from_lang_parts(
    (lang, _, Queries { highlights, .. }): &LangParts<'static>,
) -> &'static [(FormId, u8)] {
    #[rustfmt::skip]
    const PRIORITIES: &[&str] = &[
        "markup", "comment", "string", "diff", "variable", "module", "label", "character",
        "boolean", "number", "type", "attribute", "property", "function", "constant",
        "constructor", "operator", "keyword", "punctuation",
    ];
    type MemoizedForms<'a> = HashMap<&'a str, &'a [(FormId, u8)]>;

    static LISTS: LazyLock<Mutex<MemoizedForms<'static>>> = LazyLock::new(Mutex::default);
    let mut lists = LISTS.lock().unwrap();

    if let Some(forms) = lists.get(lang) {
        forms
    } else {
        let capture_names = highlights.capture_names();
        let priorities = capture_names.iter().map(|name| {
            PRIORITIES
                .iter()
                .take_while(|p| !name.starts_with(*p))
                .count() as u8
        });

        let ids = form::ids_of_non_static(
            capture_names
                .iter()
                .map(|name| name.to_string() + "." + lang),
        );
        let forms: Vec<(FormId, u8)> = ids.into_iter().zip(priorities).collect();

        lists.insert(lang, forms.leak());
        lists.get(lang).unwrap()
    }
}

#[derive(Clone, Copy)]
struct TsBuf<'a>(&'a Bytes);

impl<'a> TextProvider<&'a [u8]> for TsBuf<'a> {
    type I = std::array::IntoIter<&'a [u8], 2>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        let range = node.range();
        let buffers = self.0.buffers(range.start_byte..range.end_byte);
        buffers.to_array().into_iter()
    }
}

fn lang_parts(lang: &str) -> Result<LangParts<'static>, Text> {
    static MAPS: LazyLock<Mutex<HashMap<&str, LangParts<'static>>>> = LazyLock::new(Mutex::default);

    let mut maps = MAPS.lock().unwrap();

    Ok(if let Some(lang_parts) = maps.get(lang).copied() {
        lang_parts
    } else {
        let language: &'static Language = Box::leak(Box::new(languages::get_language(lang)?));

        let highlights = query_from_path(lang, "highlights", language)?;
        let indents = query_from_path(lang, "indents", language)?;
        let injections = query_from_path(lang, "injections", language)?;

        let queries = Queries { highlights, indents, injections };

        let lang = lang.to_string().leak();

        maps.insert(lang, (lang, language, queries));

        (lang, language, queries)
    })
}

type LangParts<'a> = (&'a str, &'a Language, Queries<'a>);

#[derive(Clone, Copy)]
struct Queries<'a> {
    highlights: &'a Query,
    indents: &'a Query,
    injections: &'a Query,
}

/// The Key for tree-sitter
fn ts_tagger() -> Tagger {
    static TAGGER: LazyLock<Tagger> = Tagger::new_static();
    *TAGGER
}

fn deoffset(ts_point: TSPoint, offset: TSPoint) -> TSPoint {
    if ts_point.row == offset.row {
        TSPoint::new(
            ts_point.row - offset.row,
            ts_point
                .column
                .checked_sub(offset.column)
                .unwrap_or_else(|| panic!("{ts_point:?}, {offset:?}")),
        )
    } else {
        TSPoint::new(ts_point.row - offset.row, ts_point.column)
    }
}

fn reoffset(ts_point: TSPoint, offset: TSPoint) -> TSPoint {
    if ts_point.row == 0 {
        TSPoint::new(ts_point.row + offset.row, ts_point.column + offset.column)
    } else {
        TSPoint::new(ts_point.row + offset.row, ts_point.column)
    }
}

fn input_edit(change: Change<&str>, bytes: &Bytes, offset: TSPoint, r_start: usize) -> InputEdit {
    let start = change.start();
    let added = change.added_end();
    let taken = change.taken_end();

    let ts_start = ts_point(start, bytes);
    let ts_taken_end = ts_point_from(taken, (ts_start.column, start), change.taken_str());
    let ts_added_end = ts_point_from(added, (ts_start.column, start), change.added_str());

    InputEdit {
        start_byte: start.byte() - r_start,
        old_end_byte: taken.byte() - r_start,
        new_end_byte: added.byte() - r_start,
        start_position: deoffset(ts_start, offset),
        old_end_position: deoffset(ts_taken_end, offset),
        new_end_position: deoffset(ts_added_end, offset),
    }
}

fn change_clips(change: Change<&str>, range: Range<usize>) -> bool {
    let start = change.start();
    let taken = change.taken_end();

    (start.byte() <= range.start && range.start < taken.byte())
        || (start.byte() < range.end && range.end <= taken.byte())
}

/// Returns a new [`Query`] for a given language and kind
///
/// If the [`Query`] in question does not exist, returns an emtpy
/// [`Query`] instead.
fn query_from_path(name: &str, kind: &str, language: &Language) -> Result<&'static Query, Text> {
    static QUERIES: LazyLock<Mutex<HashMap<PathBuf, &'static Query>>> =
        LazyLock::new(Mutex::default);

    let queries_dir = duat_core::plugin_dir("duat-treesitter")?.join("queries");

    let path = queries_dir.join(name).join(kind).with_extension("scm");

    let mut queries = QUERIES.lock().unwrap();

    Ok(if let Some(query) = queries.get(&path) {
        query
    } else {
        let Ok(mut query) = fs::read_to_string(&path) else {
            let query = Box::leak(Box::new(Query::new(language, "").unwrap()));
            queries.insert(path, query);
            return Ok(query);
        };

        let Some(first_line) = query.lines().map(String::from).next() else {
            context::warn!(target: path.to_str().unwrap(), "Query is empty");
            let query = Box::leak(Box::new(Query::new(language, "").unwrap()));
            queries.insert(path, query);
            return Ok(query);
        };

        if let Some(langs) = first_line.strip_prefix("; inherits: ") {
            for name in langs.split(',') {
                let path = queries_dir.join(name).join(kind).with_extension("scm");
                match fs::read_to_string(&path) {
                    Ok(inherited_query) => {
                        if inherited_query.is_empty() {
                            let target = path.to_str().unwrap();
                            context::warn!(target: target, "Inherited query is empty");
                        }

                        query = format!("{inherited_query}\n{query}");
                    }
                    Err(err) => context::error!("{err}"),
                }
            }
        }

        let query = Box::leak(Box::new(match Query::new(language, &query) {
            Ok(query) => query,
            Err(err) => return Err(txt!("{err}").build()),
        }));

        queries.insert(path, query);

        query
    })
}

/// Convenience methods for use of tree-sitter in [`File`]s
pub trait TsFile {
    /// The level of indentation required at a certain [`Point`]
    ///
    /// This is determined by a query, currently, it is the query
    /// located in
    /// `"{plugin_dir}/duat-treesitter/queries/{lang}/indent.scm"`
    fn ts_indent_on(&self, p: Point) -> Option<usize>;
}

impl<U: Ui> TsFile for File<U> {
    fn ts_indent_on(&self, p: Point) -> Option<usize> {
        self.read_parser(|ts: &TsParser| ts.indent_on(p, self.text().bytes(), self.print_cfg()))
            .flatten()
    }
}

/// Convenience methods for use of tree-sitter in [`Cursor`]s
pub trait TsCursor {
    /// The level of indentation required at the [`Cursor`]'s `caret`
    ///
    /// This is determined by a query, currently, it is the query
    /// located in
    /// `"{plugin_dir}/duat-treesitter/queries/{lang}/indent.scm"`
    fn ts_indent(&self) -> Option<usize>;

    /// The level of indentation required at a certain [`Point`]
    ///
    /// This is determined by a query, currently, it is the query
    /// located in
    /// `"{plugin_dir}/duat-treesitter/queries/{lang}/indent.scm"`
    fn ts_indent_on(&self, p: Point) -> Option<usize>;
}

impl<U: Ui, S> TsCursor for Cursor<'_, File<U>, U::Area, S> {
    fn ts_indent(&self) -> Option<usize> {
        self.ts_indent_on(self.caret())
    }

    fn ts_indent_on(&self, p: Point) -> Option<usize> {
        let cfg = self.cfg();

        self.read_parser(|ts: &TsParser| ts.indent_on(p, self.text().bytes(), cfg))
            .flatten()
    }
}

/// Adds two shifts together
fn add_shifts(lhs: [i32; 3], rhs: [i32; 3]) -> [i32; 3] {
    let b = lhs[0] + rhs[0];
    let c = lhs[1] + rhs[1];
    let l = lhs[2] + rhs[2];
    [b, c, l]
}

#[allow(unused)]
fn format_root(node: Node) -> Text {
    fn format_range(node: Node, builder: &mut Builder) {
        let mut first = true;
        for point in [node.start_position(), node.end_position()] {
            builder.push(txt!(
                "[punctuation.bracket.TreeView][[[coords.TreeView]{}\
             	 [punctuation.delimiter.TreeView],[] [coords.TreeView]{}\
             	 [punctuation.bracket.TreeView]]]",
                point.row,
                point.column
            ));

            if first {
                first = false;
                builder.push(txt!("[punctuation.delimiter],[] "));
            }
        }
        builder.push("\n");
    }

    fn format_node(
        node: Node,
        depth: usize,
        pars: usize,
        builder: &mut Builder,
        name: Option<&str>,
    ) {
        builder.push("  ".repeat(depth));

        if let Some(name) = name {
            builder.push(txt!("[node.field]{name}[punctuation.delimiter.TreeView]: "));
        }

        builder.push(txt!("[punctuation.bracket.TreeView]("));
        builder.push(txt!("[node.name]{}", node.grammar_name()));

        let mut cursor = node.walk();
        let named_children = node.named_children(&mut cursor);
        let len = named_children.len();

        if len == 0 {
            builder.push(txt!(
                "[punctuation.bracket.TreeView]{}[] ",
                ")".repeat(pars)
            ));
            format_range(node, builder);
        } else {
            builder.push(" ");
            format_range(node, builder);

            let mut i = 0;

            for (i, child) in named_children.enumerate() {
                let name = node.field_name_for_named_child(i as u32);
                let pars = if i == len - 1 { pars + 1 } else { 1 };
                format_node(child, depth + 1, pars, builder, name);
            }
        }
    }

    let mut cursor = node.walk();
    let mut builder = Text::builder();

    format_node(node, 0, 1, &mut builder, None);

    builder.build()
}
