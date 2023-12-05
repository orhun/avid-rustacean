use std::sync::Mutex;

use ratatui::{prelude::*, widgets::*};
use yew::Context;

use crate::{
    app::{CursorMap, Motion, TermApp},
    console_log,
    palette::GruvboxColor,
    HOST_ADDRESS,
};

static ALL_SCROLL_STATE: Mutex<Option<ScrollbarState>> = Mutex::new(None);

static SCROLL_STATE: Mutex<Option<ScrollbarState>> = Mutex::new(None);

#[derive(Debug, PartialEq)]
pub struct AllProjects {
    projects: Vec<(String, String, bool)>,
    scroll: u16,
}

#[derive(Debug)]
pub enum AllProjectsMessage {
    ProjectSummaries(Vec<(String, String)>),
}

#[derive(Debug, PartialEq)]
pub struct Project {
    name: String,
    body: String,
    scroll: u16,
}

#[derive(Debug, PartialEq)]
pub enum ProjectMessage {
    Summary(String),
}

impl Project {
    pub fn create(name: String, ctx: &Context<TermApp>, map: &mut CursorMap) -> Self {
        let cp_name = name.clone();
        ctx.link().send_future(async move {
            let body =
                match reqwest::get(format!("http{HOST_ADDRESS}/api/v1/projects/{cp_name}")).await {
                    Ok(resp) => resp.json().await.unwrap_or_default(),
                    Err(_) => String::new(),
                };
            ProjectMessage::Summary(body)
        });
        Self {
            body: String::new(),
            name,
            scroll: 0,
        }
    }

    pub fn update(&mut self, msg: ProjectMessage, map: &mut CursorMap) {
        map.clear_after(1);
        match msg {
            ProjectMessage::Summary(body) => {
                self.body = body;
            }
        }
    }

    pub fn draw(&self, mut rect: Rect, frame: &mut Frame) -> Rect {
        let widget = Paragraph::new(self.body.clone())
            .block(
                Block::new()
                    .borders(Borders::all())
                    .title(self.name.clone())
                    .title_alignment(Alignment::Center),
            )
            .scroll((self.scroll, 0));
        frame.render_widget(widget, rect);
        let mut state = SCROLL_STATE.lock().unwrap();
        state.insert(ScrollbarState::new(100));
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            rect,
            state.as_mut().unwrap(),
        );
        rect.y += rect.height;
        rect
    }
}

impl AllProjects {
    pub fn create(ctx: &Context<TermApp>, map: &mut CursorMap) -> Self {
        ALL_SCROLL_STATE
            .lock()
            .unwrap()
            .insert(ScrollbarState::default());
        ctx.link().send_future(async move {
            let projects = match reqwest::get(format!("http{HOST_ADDRESS}/api/v1/projects")).await {
                Ok(resp) => resp.json().await.unwrap_or_default(),
                Err(e) => Vec::new(),
            };
            AllProjectsMessage::ProjectSummaries(projects)
        });
        Self {
            projects: Vec::new(),
            scroll: 0,
        }
    }

    pub fn update(&mut self, msg: AllProjectsMessage, map: &mut CursorMap) {
        map.clear_after(1);
        match msg {
            AllProjectsMessage::ProjectSummaries(projects) => {
                self.projects = projects.into_iter().map(|(n, s)| (n, s, false)).collect();
            }
        }
        for (title, _, _) in self.projects.iter() {
            map.append_and_push(title.clone());
        }
    }

    pub fn handle_motion(&mut self, motion: Motion, map: &CursorMap) {
        console_log(format!(
            "Handling motion in projects. New position: {:?}",
            map.get_position()
        ));
        match map.get_position() {
            (0, y) if y > 0 && y <= self.projects.len() => {
                self.projects
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, (_, _, sel))| *sel = i + 1 == y);
            }
            _ => {
                self.projects
                    .iter_mut()
                    .for_each(|(_, _, sel)| *sel = false);
            }
        }
    }

    pub fn draw(&self, mut rect: Rect, frame: &mut Frame) -> Rect {
        let widget = Paragraph::new(
            self.projects
                .iter()
                .map(|(s, _, sel)| get_line(s, *sel))
                .collect::<Vec<_>>(),
        )
        .block(Block::new().borders(Borders::all()))
        .scroll((self.scroll, 0));
        frame.render_widget(widget, rect);
        let mut state = ALL_SCROLL_STATE.lock().unwrap();
        state.insert(ScrollbarState::new(100));
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            rect,
            state.as_mut().unwrap(),
        );
        rect.y += rect.height;
        rect
    }
}

fn get_line(s: &str, selected: bool) -> Line {
    let style = if selected {
        GruvboxColor::green().full_style(GruvboxColor::dark_3())
    } else {
        GruvboxColor::default_style()
    };
    Line::styled(s, style)
}
