use crate::comments::*;
use crate::regexes::*;
use crate::Cli;

const WRAP: usize = 80;

pub fn needs_wrap(file: &str) -> bool {
    file.lines().any(|l| l.len() > WRAP)
}

pub fn line_needs_wrap(line: &str) -> bool {
    line.len() > WRAP
}

pub fn find_wrap_point(line: &str) -> Option<usize> {
    let mut wrap_point: Option<usize> = None;
    let mut after_char = false;
    let mut prev_char: Option<char> = None;
    for i in 0..WRAP {
        if line.chars().nth(i) == Some(' ') && prev_char != Some('\\') {
            if after_char {
                wrap_point = Some(i);
            }
        } else if line.chars().nth(i) != Some('%') {
            after_char = true;
        }
        prev_char = line.chars().nth(i)
    }
    wrap_point
}

pub fn wrap_line(line: &str, args: &Cli) -> String {
    let mut remaining_line = line.to_string();
    let mut new_line = "".to_string();
    let mut can_wrap = true;
    while line_needs_wrap(&remaining_line) && can_wrap {
        let wrap_point = find_wrap_point(&remaining_line);
        let comment_index = find_comment_index(&remaining_line);
        match wrap_point {
            Some(p) => {
                let line_start = match comment_index {
                    Some(c) => {
                        if p > c {
                            "%"
                        } else {
                            ""
                        }
                    }
                    None => "",
                };
                new_line.push_str(&remaining_line[0..p]);
                new_line.push('\n');
                remaining_line =
                    remaining_line[p..remaining_line.len()].to_string();
                remaining_line.insert_str(0, line_start);
            }
            None => {
                can_wrap = false;
                if args.debug {
                    eprintln!("long line cannot be wrapped!");
                    eprintln!("{}", remaining_line);
                }
            }
        }
    }
    new_line.push_str(&remaining_line);
    new_line
}

pub fn wrap(file: &str, args: &Cli) -> String {
    let mut new_file = "".to_string();
    let mut verbatim_count = 0;
    for line in file.lines() {
        if RE_VERBATIM_BEGIN.is_match(line) {
            verbatim_count += 1;
        }
        if line_needs_wrap(line) && verbatim_count == 0 {
            let new_line = wrap_line(line, args);
            new_file.push_str(&new_line);
        } else {
            new_file.push_str(line);
        }
        new_file.push('\n');
        if RE_VERBATIM_BEGIN.is_match(line) {
            verbatim_count += 1;
        }
    }
    new_file
}

#[cfg(test)]
#[test]
fn test_wrap_line() {
    let args = Cli::new();
    // no comment
    let s_in = "This line is too long because it has more than eighty characters inside it. \
        Therefore it should be split.";
    let s_out = "This line is too long because it has more than eighty characters inside it.\n \
        Therefore it should be split.";
    assert_eq!(wrap_line(s_in, &args), s_out);
    // break before comment
    let s_in = "This line is too long because it has more than eighty characters inside it. \
        Therefore it % should be split.";
    let s_out = "This line is too long because it has more than eighty characters inside it.\n \
        Therefore it % should be split.";
    assert_eq!(wrap_line(s_in, &args), s_out);
    // break after comment
    let s_in = "This line is too long because % it has more than eighty characters inside it. \
        Therefore it should be split.";
    let s_out = "This line is too long because % it has more than eighty characters inside it.\n\
        % Therefore it should be split.";
    assert_eq!(wrap_line(s_in, &args), s_out);
    // leading spaces
    let s_in = "    Thislineistoolongbecauseithasmorethaneightycharactersinsideiteventhoughitstartswithspaces. \
        Thereforeitshouldbesplit.";
    let s_out = s_in;
    assert_eq!(wrap_line(s_in, &args), s_out);
}
