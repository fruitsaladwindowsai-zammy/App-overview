// App model and Alfred-style search matching algorithm from Savanna

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub target: String,
    #[allow(dead_code)]
    pub is_uwp: bool,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SearchFields {
    pub name_folded: String,
    pub name_words: Vec<String>,
    pub initials: String,
    pub id_words: Vec<String>,
    pub tag_words: Vec<String>,
}

pub fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

pub fn words(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut prev_char: Option<char> = None;

    for c in text.chars() {
        if let Some(prev) = prev_char {
            if prev.is_lowercase() && c.is_uppercase() {
                if !current_word.is_empty() {
                    result.push(current_word.to_lowercase());
                    current_word.clear();
                }
            }
        }

        if c.is_alphanumeric() {
            current_word.push(c);
        } else if !current_word.is_empty() {
            result.push(current_word.to_lowercase());
            current_word.clear();
        }
        prev_char = Some(c);
    }

    if !current_word.is_empty() {
        result.push(current_word.to_lowercase());
    }

    result
}

impl AppEntry {
    pub fn search_fields(&self) -> SearchFields {
        let name_folded = fold(&self.name);
        let name_words = words(&self.name);
        let initials = name_words
            .iter()
            .filter_map(|w| w.chars().next())
            .collect();
        let id_words = words(&self.id);
        let tag_words = self
            .keywords
            .iter()
            .flat_map(|k| words(k))
            .collect();

        SearchFields {
            name_folded,
            name_words,
            initials,
            id_words,
            tag_words,
        }
    }
}

fn any_prefix(list: &[String], term: &str) -> Option<usize> {
    for (i, word) in list.iter().enumerate() {
        if word.starts_with(term) {
            return Some(i);
        }
    }
    None
}

pub fn score_term(f: &SearchFields, term: &str) -> i32 {
    if let Some(idx) = any_prefix(&f.name_words, term) {
        return if idx == 0 { 1200 } else { 1000 };
    }
    if term.len() >= 2 && f.initials.starts_with(term) {
        return 900;
    }
    if any_prefix(&f.id_words, term).is_some() {
        return 600;
    }
    if term.len() >= 3 && f.name_folded.contains(term) {
        return 500;
    }
    if any_prefix(&f.tag_words, term).is_some() {
        return 300;
    }
    -1
}

pub fn score_entry(entry: &AppEntry, query: &str) -> i32 {
    let q = fold(query).trim().to_string();
    if q.is_empty() {
        return 0;
    }

    let f = entry.search_fields();
    let terms: Vec<&str> = q.split_whitespace().collect();
    let mut score = 0;

    for term in &terms {
        let s = score_term(&f, term);
        if s < 0 {
            return -1;
        }
        score += s;
    }

    // Whole query starting the name beats everything; shorter names preferred
    if f.name_folded.starts_with(&q) {
        score += 5000;
    }

    score - (f.name_folded.len() as i32)
}

pub fn per_page(columns: usize, rows: usize) -> usize {
    columns.max(1) * rows.max(1)
}

pub fn page_count(total_items: usize, columns: usize, rows: usize) -> usize {
    let pp = per_page(columns, rows);
    if pp == 0 || total_items == 0 {
        1
    } else {
        (total_items + pp - 1) / pp
    }
}

pub fn move_selection(
    current_index: usize,
    direction: MoveDirection,
    total_count: usize,
    columns: usize,
    rows: usize,
) -> usize {
    if total_count == 0 {
        return 0;
    }
    let last = total_count - 1;
    let current = current_index.min(last);
    let pp = per_page(columns, rows);
    let pages = page_count(total_count, columns, rows);

    let page = current / pp;
    let local = current % pp;
    let row = local / columns;
    let col = local % columns;

    let target = match direction {
        MoveDirection::Right => {
            if col < columns - 1 && current < last {
                current + 1
            } else if page < pages - 1 {
                let next_idx = (page + 1) * pp + row * columns;
                next_idx.min(last)
            } else {
                current
            }
        }
        MoveDirection::Left => {
            if col > 0 {
                current - 1
            } else if page > 0 {
                (page - 1) * pp + row * columns + (columns - 1)
            } else {
                current
            }
        }
        MoveDirection::Down => {
            if row < rows - 1 && current + columns <= last {
                current + columns
            } else {
                current
            }
        }
        MoveDirection::Up => {
            if row > 0 {
                current.saturating_sub(columns)
            } else {
                current
            }
        }
    };

    target.min(last)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}
