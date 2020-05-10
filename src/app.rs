use log::info;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use web_sys::HtmlInputElement;
use yew::{
    format::Json,
    prelude::*,
    services::storage::{Area, StorageService},
};

pub struct App {
    link: ComponentLink<Self>,
    storage: Option<StorageService>,
    todos: Vec<Todo>,
    filter: Filter,
    value: String,
    new_input: NodeRef,
    edit_input: NodeRef,
    focus_edit: bool, // Focus edit input on next tick.
}

#[derive(Serialize, Deserialize)]
struct Todo {
    text: String,
    completed: bool,
    #[serde(skip)]
    editing: bool,
}

impl Todo {
    fn new(text: &str) -> Todo {
        Todo {
            text: text.to_owned(),
            completed: false,
            editing: false,
        }
    }
}

pub enum Msg {
    Nop,
    Update(String),
    Create,
    Delete(usize),
    Toggle(usize),
    ToggleAll,
    ClearCompleted,
    StartEdit(usize),
    UpdateItem(usize, String),
    CompleteEdit(usize),
    Filter(Filter),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, Display)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn accept(self, todo: &Todo) -> bool {
        match (self, todo.completed) {
            (Filter::All, _) => true,
            (Filter::Active, false) => true,
            (Filter::Completed, true) => true,
            _ => false,
        }
    }
}

const STORAGE_KEY: &str = "todos";

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).ok();
        let todos = if let Some(storage) = storage.as_ref() {
            if let Json(Ok(todos)) = storage.restore(STORAGE_KEY) {
                todos
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        App {
            link,
            storage,
            todos,
            filter: Filter::All,
            value: String::new(),
            new_input: NodeRef::default(),
            edit_input: NodeRef::default(),
            focus_edit: false,
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Nop => (),
            Msg::Update(s) => self.value = s,
            Msg::Create => {
                let text = self.value.trim();
                if !text.is_empty() {
                    info!("Create {}.", text);
                    self.todos.push(Todo::new(text));
                    self.value.clear();
                }
            }
            Msg::Delete(index) => {
                info!("Delete #{}.", index);
                self.todos.remove(index);
            }
            Msg::Toggle(index) => {
                info!("Toggle #{}.", index);
                let todo = &mut self.todos[index];
                todo.completed = !todo.completed;
            }
            Msg::ToggleAll => {
                info!("Toggle all.");
                let completed = !self.all_completed();
                for todo in &mut self.todos {
                    todo.completed = completed;
                }
            }
            Msg::ClearCompleted => {
                info!("Clear completed.");
                self.todos.retain(|todo| !todo.completed);
            }
            Msg::StartEdit(index) => {
                info!("Start editing #{}.", index);
                self.todos[index].editing = true;
                self.focus_edit = true;
            }
            Msg::UpdateItem(index, value) => self.todos[index].text = value,
            Msg::CompleteEdit(index) => {
                info!("Complete editing #{}.", index);
                self.todos[index].editing = false;
            },
            Msg::Filter(f) => {
                info!("Filter {}", f);
                self.filter = f;
            }
        }
        if let Some(storage) = self.storage.as_mut() {
            storage.store(STORAGE_KEY, Json(&self.todos));
        }
        true
    }
    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.new_input
                .cast::<HtmlInputElement>()
                .unwrap()
                .focus()
                .unwrap();
        }
        if self.focus_edit {
            self.focus_edit = false;
            self.edit_input
                .cast::<HtmlInputElement>()
                .unwrap()
                .focus()
                .unwrap();
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        html! {
            <>
                <section class="todoapp">
                    <header class="header">
                        <h1>{"todos"}</h1>
                        <input
                            class="new-todo"
                            placeholder="What needs to be done?"
                            ref=self.new_input.clone(),
                            value=&self.value,
                            oninput=self.link.callback(|e:InputData|Msg::Update(e.value))
                            onkeypress=self.link.callback(|e:KeyboardEvent| {
                                if e.key() == "Enter" {Msg::Create} else {Msg::Nop}
                            }) />
                    </header>
                    {if !self.todos.is_empty() {
                        html! {
                            <>
                                {self.view_main()}
                                {self.view_footer()}
                            </>
                        }
                    } else {
                        html!{}
                    }}
                </section>
                <footer class="info">
                    <p>{"Double-click to edit a todo"}</p>
                    <p>{"Created by "}<a href="https://github.com/kaoet">{"Kaoet"}</a></p>
                    <p>{"Part of "}<a href="http://todomvc.com">{"TodoMVC"}</a></p>
                </footer>
            </>
        }
    }
}

impl App {
    fn view_main(&self) -> Html {
        html! {
            <section class="main">
                {if !self.todos.is_empty() {
                    html!{
                        <>
                            <input
                                id="toggle-all"
                                class="toggle-all"
                                type="checkbox"
                                checked=self.all_completed()
                                onclick=self.link.callback(|_|Msg::ToggleAll) />
                            <label for="toggle-all">{"Mark all as complete"}</label>
                        </>
                    }
                } else {
                    html!{}
                }}
                <ul class="todo-list">
                    {for (0..self.todos.len()).map(|index|self.view_todo(index))}
                </ul>
            </section>
        }
    }

    fn view_todo(&self, index: usize) -> Html {
        let todo = &self.todos[index];
        if self.filter.accept(todo) {
            html! {
                <li class={if todo.editing{"editing"}else if todo.completed{"completed"} else {""}}>
                    <div class="view">
                        <input class="toggle" type="checkbox" checked=todo.completed onchange={self.link.callback(move |_|Msg::Toggle(index))} />
                        <label ondoubleclick=self.link.callback(move |_|Msg::StartEdit(index))>{&todo.text}</label>
                        <button class="destroy" onclick=self.link.callback(move |_|Msg::Delete(index))></button>
                    </div>
                    {if todo.editing {
                        html! {
                            <input
                                class="edit"
                                value=&todo.text
                                ref=self.edit_input.clone()
                                oninput=self.link.callback(move |e:InputData|Msg::UpdateItem(index, e.value))
                                onblur=self.link.callback(move |_|Msg::CompleteEdit(index))
                                onkeypress=self.link.callback(move |e:KeyboardEvent| {
                                    if e.key() == "Enter" {Msg::CompleteEdit(index)} else {Msg::Nop}
                                }) />
                        }
                    } else {
                        html!{}
                    }}
                </li>
            }
        } else {
            html! {}
        }
    }

    fn view_footer(&self) -> Html {
        let completed = self.todos.iter().filter(|t| t.completed).count();
        let active = self.todos.len() - completed;
        html! {
            <footer class="footer">
                <span class="todo-count"><strong>{active}</strong>{" item(s) left"}</span>
                <ul class="filters">
                {for Filter::iter().map(|f| html!{
                    <li>
                        <a
                            class=if self.filter == f{"selected"}else{""} href="javascript:"
                            onclick=self.link.callback(move |_|Msg::Filter(f))>
                            {f}
                        </a>
                    </li>
                })}
                </ul>
                {if completed > 0 {
                    html! {<button class="clear-completed" onclick=self.link.callback(|_|Msg::ClearCompleted)>{"Clear completed("}{completed}{")"}</button>}
                } else {
                    html!{}
                }}
            </footer>
        }
    }

    fn all_completed(&self) -> bool {
        self.todos.iter().all(|todo| todo.completed)
    }
}
