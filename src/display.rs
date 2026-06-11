use chrono::{TimeZone, Utc};
use colored::Colorize;
use terminal_size::{terminal_size, Width};

use crate::api::{Item, User};

pub fn term_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80)
}

pub fn time_ago(timestamp: u64) -> String {
    let now = Utc::now().timestamp() as u64;
    if timestamp >= now {
        return "just now".to_string();
    }
    let diff = now - timestamp;
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3_600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86_400 {
        format!("{}h ago", diff / 3_600)
    } else if diff < 2_592_000 {
        format!("{}d ago", diff / 86_400)
    } else {
        Utc.timestamp_opt(timestamp as i64, 0)
            .single()
            .map(|dt| dt.format("%b %d").to_string())
            .unwrap_or_default()
    }
}

pub fn render_html(html: &str, width: usize) -> String {
    let text = html2text::from_read(html.as_bytes(), width).unwrap_or_else(|_| html.to_string());
    let mut result = String::new();
    let mut prev_empty = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_empty {
                result.push('\n');
                prev_empty = true;
            }
        } else {
            result.push_str(trimmed);
            result.push('\n');
            prev_empty = false;
        }
    }
    result.trim().to_string()
}

pub fn story_domain(url: &Option<String>) -> String {
    match url {
        None => String::new(),
        Some(url) => {
            let after_proto = url.find("://").map(|i| i + 3).unwrap_or(0);
            let rest = &url[after_proto..];
            let end = rest.find('/').unwrap_or(rest.len());
            let domain = &rest[..end];
            domain.strip_prefix("www.").unwrap_or(domain).to_string()
        }
    }
}

pub fn print_story_list(stories: &[Item], start: usize) {
    for (i, story) in stories.iter().enumerate() {
        let num = format!("{}.", start + i);
        print!("{} ", num.color(colored::Color::BrightBlack));

        let title = story.title.as_deref().unwrap_or("(no title)");
        print!("{}", title.color(colored::Color::White).bold());

        let domain = story_domain(&story.url);
        if !domain.is_empty() {
            print!(" {}", format!("({})", domain).dimmed());
        }
        println!();

        if let Some(ref url) = story.url {
            println!("   {}", url.dimmed());
        }

        let points = story.score.unwrap_or(0);
        let by = story.by.as_deref().unwrap_or("unknown");
        let time = story.time.map(time_ago).unwrap_or_default();
        let comments = story.descendants.unwrap_or(0);

        println!(
            "   {} by {} {} | {} comments | id {}",
            format!("{} points", points).dimmed(),
            by.dimmed(),
            time.dimmed(),
            comments.to_string().dimmed(),
            story.id.to_string().dimmed()
        );
    }
}

pub fn print_story_detail(story: &Item, comments: &[Item], width: usize) {
    let title = story.title.as_deref().unwrap_or("(no title)");
    println!("{}", title.color(colored::Color::White).bold());

    if let Some(ref url) = story.url {
        println!("{}", url.dimmed());
    }

    let points = story.score.unwrap_or(0);
    let by = story.by.as_deref().unwrap_or("unknown");
    let time = story.time.map(time_ago).unwrap_or_default();
    let descendants = story.descendants.unwrap_or(0);
    println!(
        "{} by {} {} | {} comments",
        format!("{} points", points).dimmed(),
        by.dimmed(),
        time.dimmed(),
        descendants.to_string().dimmed()
    );

    if let Some(ref text) = story.text {
        if !text.is_empty() {
            println!();
            println!("{}", render_html(text, width));
        }
    }

    if !comments.is_empty() {
        println!();
        println!("{}", "-".repeat(width).dimmed());
        println!();

        let kids = story.kids.as_deref().unwrap_or(&[]);
        for kid_id in kids {
            if let Some(comment) = comments.iter().find(|c| c.id == *kid_id) {
                print_comment_tree(comment, comments, 0, width);
            }
        }
    }
}

pub fn print_comment_tree(comment: &Item, all_comments: &[Item], depth: usize, width: usize) {
    if comment.dead.unwrap_or(false) || comment.deleted.unwrap_or(false) {
        return;
    }

    let indent = " ".repeat(depth * 2);

    let by = comment.by.as_deref().unwrap_or("unknown");
    let time = comment.time.map(time_ago).unwrap_or_default();
    println!("{}{} {}", indent, by.dimmed(), time.dimmed());

    if let Some(ref text) = comment.text {
        if !text.is_empty() {
            let avail = width.saturating_sub(depth * 2);
            let rendered = render_html(text, avail);
            for line in rendered.lines() {
                println!("{}{}", indent, line);
            }
        }
    }

    let kids = comment.kids.as_deref().unwrap_or(&[]);
    for kid_id in kids {
        if let Some(child) = all_comments.iter().find(|c| c.id == *kid_id) {
            print_comment_tree(child, all_comments, depth + 1, width);
        }
    }

    if depth == 0 {
        println!();
    }
}

pub fn print_story_json(story: &Item, comments: &[Item]) {
    let comment_values: Vec<serde_json::Value> = comments
        .iter()
        .map(|c| {
            let mut obj = serde_json::Map::new();
            obj.insert("id".into(), serde_json::Value::Number(c.id.into()));
            if let Some(ref by) = c.by {
                obj.insert("by".into(), serde_json::Value::String(by.clone()));
            }
            if let Some(ref text) = c.text {
                obj.insert("text".into(), serde_json::Value::String(text.clone()));
            }
            if let Some(time) = c.time {
                obj.insert("time".into(), serde_json::Value::Number(time.into()));
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), serde_json::Value::Number(story.id.into()));
    if let Some(ref title) = story.title {
        obj.insert("title".into(), serde_json::Value::String(title.clone()));
    }
    if let Some(ref url) = story.url {
        obj.insert("url".into(), serde_json::Value::String(url.clone()));
    }
    if let Some(ref by) = story.by {
        obj.insert("by".into(), serde_json::Value::String(by.clone()));
    }
    if let Some(score) = story.score {
        obj.insert("score".into(), serde_json::Value::Number(score.into()));
    }
    if let Some(time) = story.time {
        obj.insert("time".into(), serde_json::Value::Number(time.into()));
    }
    if let Some(descendants) = story.descendants {
        obj.insert(
            "descendants".into(),
            serde_json::Value::Number(descendants.into()),
        );
    }
    if let Some(ref text) = story.text {
        obj.insert("text".into(), serde_json::Value::String(text.clone()));
    }
    obj.insert(
        "comments".into(),
        serde_json::Value::Array(comment_values),
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap()
    );
}

pub fn print_story_list_json(stories: &[Item]) {
    let values: Vec<serde_json::Value> = stories
        .iter()
        .map(|story| {
            let mut obj = serde_json::Map::new();
            obj.insert("id".into(), serde_json::Value::Number(story.id.into()));
            if let Some(ref title) = story.title {
                obj.insert("title".into(), serde_json::Value::String(title.clone()));
            }
            if let Some(ref url) = story.url {
                obj.insert("url".into(), serde_json::Value::String(url.clone()));
            }
            if let Some(ref by) = story.by {
                obj.insert("by".into(), serde_json::Value::String(by.clone()));
            }
            if let Some(score) = story.score {
                obj.insert("score".into(), serde_json::Value::Number(score.into()));
            }
            if let Some(time) = story.time {
                obj.insert("time".into(), serde_json::Value::Number(time.into()));
            }
            if let Some(descendants) = story.descendants {
                obj.insert(
                    "descendants".into(),
                    serde_json::Value::Number(descendants.into()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Array(values)).unwrap()
    );
}

pub fn print_user(user: &User) {
    println!(
        "User: {}",
        user.id.color(colored::Color::White).bold()
    );
    if let Some(karma) = user.karma {
        println!("Karma: {}", karma);
    }
    if let Some(dt) = Utc.timestamp_opt(user.created as i64, 0).single() {
        println!("Created: {}", dt.format("%Y-%m-%d"));
    }
    if let Some(ref about) = user.about {
        if !about.is_empty() {
            let width = term_width();
            println!("About: {}", render_html(about, width));
        }
    }
}

pub fn print_user_json(user: &User) {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), serde_json::Value::String(user.id.clone()));
    if let Some(karma) = user.karma {
        obj.insert("karma".into(), serde_json::Value::Number(karma.into()));
    }
    obj.insert(
        "created".into(),
        serde_json::Value::Number(user.created.into()),
    );
    if let Some(ref about) = user.about {
        obj.insert("about".into(), serde_json::Value::String(about.clone()));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap()
    );
}
