use similar::{Algorithm, ChangeTag, DiffOp, TextDiff};

#[derive(Clone, Debug, PartialEq)]
pub enum DiffKind {
    Equal,
    LeftOnly,
    RightOnly,
    Modified,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlinePart {
    pub text: String,
    pub highlighted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiffRow {
    pub left_line_num: Option<usize>,
    pub right_line_num: Option<usize>,
    pub left_text: Option<String>,
    pub right_text: Option<String>,
    pub kind: DiffKind,
    pub left_inline: Vec<InlinePart>,
    pub right_inline: Vec<InlinePart>,
    /// 0-indexed insertion point in the right file.
    pub right_pos: usize,
    /// 0-indexed insertion point in the left file.
    pub left_pos: usize,
    /// Consecutive non-Equal rows sharing a block_id belong to the same change group.
    pub block_id: usize,
}

pub fn compute_diff(old: &str, new: &str, ignore_whitespace: bool) -> Vec<DiffRow> {
    let old_orig: Vec<&str> = old.lines().collect();
    let new_orig: Vec<&str> = new.lines().collect();

    let (old_cmp, new_cmp) = normalized_lines(&old_orig, &new_orig, ignore_whitespace);
    let old_cmp_ref: Vec<&str> = old_cmp.iter().map(|s| s.as_str()).collect();
    let new_cmp_ref: Vec<&str> = new_cmp.iter().map(|s| s.as_str()).collect();

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Histogram)
        .diff_slices(&old_cmp_ref, &new_cmp_ref);

    let mut rows: Vec<DiffRow> = Vec::new();
    let mut block_id: usize = 0;
    let mut last_was_equal = true;

    for op in diff.ops() {
        match *op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for i in 0..len {
                    rows.push(equal_row(
                        &old_orig,
                        &new_orig,
                        old_index + i,
                        new_index + i,
                        block_id,
                    ));
                }
                last_was_equal = true;
            }
            DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => {
                if last_was_equal {
                    block_id += 1;
                    last_was_equal = false;
                }
                for i in 0..old_len {
                    rows.push(left_only_row(&old_orig, old_index + i, new_index, block_id));
                }
            }
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                if last_was_equal {
                    block_id += 1;
                    last_was_equal = false;
                }
                for i in 0..new_len {
                    rows.push(right_only_row(
                        &new_orig,
                        new_index + i,
                        old_index,
                        block_id,
                    ));
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                if last_was_equal {
                    block_id += 1;
                    last_was_equal = false;
                }
                let max_len = old_len.max(new_len);
                for i in 0..max_len {
                    rows.push(replace_row(
                        &old_orig, &new_orig, old_index, old_len, new_index, new_len, i, block_id,
                    ));
                }
            }
        }
    }

    rows
}

/// Returns row indices to show in "diffs only" mode: each change plus `context` equal lines around it.
/// Adjacent windows that overlap are merged.
pub fn visible_ranges(rows: &[DiffRow], context: usize) -> Vec<std::ops::Range<usize>> {
    let changes: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind != DiffKind::Equal)
        .map(|(i, _)| i)
        .collect();

    if changes.is_empty() {
        return vec![];
    }

    let n = rows.len();
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    let mut i = 0;

    while i < changes.len() {
        let start = changes[i].saturating_sub(context);
        let mut end = (changes[i] + context + 1).min(n);

        while i + 1 < changes.len() && changes[i + 1].saturating_sub(context) <= end {
            i += 1;
            end = (changes[i] + context + 1).min(n);
        }

        ranges.push(start..end);
        i += 1;
    }

    ranges
}

pub fn copy_row_right_to_left(row: &DiffRow, left: &[String]) -> Vec<String> {
    let mut result = left.to_vec();
    match row.kind {
        DiffKind::RightOnly => {
            let pos = row.left_pos.min(result.len());
            result.insert(pos, row.right_text.clone().unwrap_or_default());
        }
        DiffKind::Modified => {
            if let (Some(n), Some(text)) = (row.left_line_num, &row.right_text)
                && n - 1 < result.len()
            {
                result[n - 1] = text.clone();
            }
        }
        _ => {}
    }
    result
}

pub fn copy_row_left_to_right(row: &DiffRow, right: &[String]) -> Vec<String> {
    let mut result = right.to_vec();
    match row.kind {
        DiffKind::LeftOnly => {
            let pos = row.right_pos.min(result.len());
            result.insert(pos, row.left_text.clone().unwrap_or_default());
        }
        DiffKind::Modified => {
            if let (Some(n), Some(text)) = (row.right_line_num, &row.left_text)
                && n - 1 < result.len()
            {
                result[n - 1] = text.clone();
            }
        }
        _ => {}
    }
    result
}

pub fn copy_block_right_to_left(bid: usize, rows: &[DiffRow], left: &[String]) -> Vec<String> {
    let block: Vec<&DiffRow> = rows
        .iter()
        .filter(|r| r.block_id == bid && r.kind != DiffKind::Equal)
        .collect();

    let left_start = block
        .iter()
        .filter_map(|r| r.left_line_num)
        .min()
        .map(|n| n - 1);
    let left_end = block.iter().filter_map(|r| r.left_line_num).max();
    let insert_pos = block.first().map(|r| r.left_pos).unwrap_or(left.len());
    let replacement: Vec<String> = block.iter().filter_map(|r| r.right_text.clone()).collect();

    splice_or_insert(left, left_start, left_end, insert_pos, replacement)
}

pub fn copy_block_left_to_right(bid: usize, rows: &[DiffRow], right: &[String]) -> Vec<String> {
    let block: Vec<&DiffRow> = rows
        .iter()
        .filter(|r| r.block_id == bid && r.kind != DiffKind::Equal)
        .collect();

    let right_start = block
        .iter()
        .filter_map(|r| r.right_line_num)
        .min()
        .map(|n| n - 1);
    let right_end = block.iter().filter_map(|r| r.right_line_num).max();
    let insert_pos = block.first().map(|r| r.right_pos).unwrap_or(right.len());
    let replacement: Vec<String> = block.iter().filter_map(|r| r.left_text.clone()).collect();

    splice_or_insert(right, right_start, right_end, insert_pos, replacement)
}

/// Returns normalized lines for diffing (collapse whitespace when requested).
fn normalized_lines(
    orig_old: &[&str],
    orig_new: &[&str],
    ignore_whitespace: bool,
) -> (Vec<String>, Vec<String>) {
    if ignore_whitespace {
        let norm = |lines: &[&str]| -> Vec<String> {
            lines
                .iter()
                .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
                .collect()
        };
        (norm(orig_old), norm(orig_new))
    } else {
        (
            orig_old.iter().map(|s| s.to_string()).collect(),
            orig_new.iter().map(|s| s.to_string()).collect(),
        )
    }
}

fn equal_row(old: &[&str], new: &[&str], oi: usize, ni: usize, block_id: usize) -> DiffRow {
    DiffRow {
        left_line_num: Some(oi + 1),
        right_line_num: Some(ni + 1),
        left_text: old.get(oi).map(|s| s.to_string()),
        right_text: new.get(ni).map(|s| s.to_string()),
        kind: DiffKind::Equal,
        left_inline: vec![],
        right_inline: vec![],
        right_pos: ni,
        left_pos: oi,
        block_id,
    }
}

fn left_only_row(old: &[&str], oi: usize, new_insert: usize, block_id: usize) -> DiffRow {
    DiffRow {
        left_line_num: Some(oi + 1),
        right_line_num: None,
        left_text: old.get(oi).map(|s| s.to_string()),
        right_text: None,
        kind: DiffKind::LeftOnly,
        left_inline: vec![],
        right_inline: vec![],
        right_pos: new_insert,
        left_pos: oi,
        block_id,
    }
}

fn right_only_row(new: &[&str], ni: usize, old_insert: usize, block_id: usize) -> DiffRow {
    DiffRow {
        left_line_num: None,
        right_line_num: Some(ni + 1),
        left_text: None,
        right_text: new.get(ni).map(|s| s.to_string()),
        kind: DiffKind::RightOnly,
        left_inline: vec![],
        right_inline: vec![],
        right_pos: ni,
        left_pos: old_insert,
        block_id,
    }
}

#[allow(clippy::too_many_arguments)]
fn replace_row(
    old: &[&str],
    new: &[&str],
    old_index: usize,
    old_len: usize,
    new_index: usize,
    new_len: usize,
    i: usize,
    block_id: usize,
) -> DiffRow {
    let has_old = i < old_len;
    let has_new = i < new_len;

    let left_text = has_old
        .then(|| old.get(old_index + i).map(|s| s.to_string()))
        .flatten();
    let right_text = has_new
        .then(|| new.get(new_index + i).map(|s| s.to_string()))
        .flatten();

    let (kind, left_inline, right_inline) = if has_old && has_new {
        let (li, ri) = compute_inline(
            old.get(old_index + i).copied().unwrap_or(""),
            new.get(new_index + i).copied().unwrap_or(""),
        );
        (DiffKind::Modified, li, ri)
    } else if has_old {
        (DiffKind::LeftOnly, vec![], vec![])
    } else {
        (DiffKind::RightOnly, vec![], vec![])
    };

    DiffRow {
        left_line_num: has_old.then_some(old_index + i + 1),
        right_line_num: has_new.then_some(new_index + i + 1),
        left_text,
        right_text,
        kind,
        left_inline,
        right_inline,
        right_pos: new_index + i.min(new_len - 1),
        left_pos: old_index + i.min(old_len - 1),
        block_id,
    }
}

fn compute_inline(old: &str, new: &str) -> (Vec<InlinePart>, Vec<InlinePart>) {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_words(old, new);

    let mut left_parts: Vec<InlinePart> = Vec::new();
    let mut right_parts: Vec<InlinePart> = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change.value().to_string();
        match change.tag() {
            ChangeTag::Equal => {
                left_parts.push(InlinePart {
                    text: text.clone(),
                    highlighted: false,
                });
                right_parts.push(InlinePart {
                    text,
                    highlighted: false,
                });
            }
            ChangeTag::Delete => left_parts.push(InlinePart {
                text,
                highlighted: true,
            }),
            ChangeTag::Insert => right_parts.push(InlinePart {
                text,
                highlighted: true,
            }),
        }
    }

    (left_parts, right_parts)
}

/// Either splices `replacement` into `start..end`, or inserts at `insert_pos` when there
/// is no range to replace (pure insertion from the other side).
fn splice_or_insert(
    lines: &[String],
    start: Option<usize>,
    end: Option<usize>,
    insert_pos: usize,
    replacement: Vec<String>,
) -> Vec<String> {
    let mut result = lines.to_vec();
    match (start, end) {
        (Some(s), Some(e)) => {
            result.splice(s..e, replacement);
        }
        _ => {
            let pos = insert_pos.min(result.len());
            for (i, line) in replacement.into_iter().enumerate() {
                result.insert(pos + i, line);
            }
        }
    }
    result
}
