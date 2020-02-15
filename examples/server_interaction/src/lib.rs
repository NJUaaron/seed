//! [https://rustwasm.github.io/wasm-bindgen/examples/fetch.html](https://rustwasm.github.io/wasm-bindgen/examples/fetch.html)
//! [https://serde.rs/](https://serde.rs/)

#![allow(clippy::large_enum_variant)]

use seed::{fetch::*, prelude::*, *};
use serde::{Deserialize, Serialize};

const REPOSITORY_URL: &str = "https://api.github.com/repos/seed-rs/seed/branches/master";
const CONTACT_URL: &str = "https://infinitea.herokuapp.com/api/contact";

#[derive(Serialize)]
struct SendMessageRequestBody {
    pub name: String,
    pub email: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct SendMessageResponseBody {
    pub success: bool,
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug, Serialize, Deserialize)]
struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    pub sha: String,
}

struct Model {
    branch: Branch,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            branch: Branch {
                name: "Loading...".into(),
                commit: Commit {
                    sha: "Loading...".into(),
                },
            },
        }
    }
}

// ------ ------
//  After Mount
// ------ ------

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders.perform_cmd(fetch_repository_info());
    AfterMount::default()
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    RepositoryInfoFetched(Result<Branch, FetchError>),
    SendMessage,
    MessageSent(Result<SendMessageResponseBody, FetchError>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::RepositoryInfoFetched(Ok(branch)) => model.branch = branch,

        Msg::RepositoryInfoFetched(Err(fetch_error)) => {
            error!(format!(
                "Fetch error - Fetching repository info failed - {:#?}",
                fetch_error
            ));
            orders.skip();
        }

        Msg::SendMessage => {
            orders.skip().perform_cmd(send_message());
        }

        Msg::MessageSent(Ok(response_data)) => {
            log!(format!("Response data: {:#?}", response_data));
            orders.skip();
        }

        Msg::MessageSent(Err(fail_reason)) => {
            error!(format!(
                "Fetch error - Sending message failed - {:#?}",
                fail_reason
            ));
            orders.skip();
        }
    }
}

async fn fetch_repository_info() -> Msg {
    // TODO chain?
    let response = match fetch(REPOSITORY_URL).await {
        Err(err) => return Msg::RepositoryInfoFetched(Err(err)),
        Ok(response) => response,
    };
    let branch = match response.json().await {
        Err(err) => return Msg::RepositoryInfoFetched(Err(err)),
        Ok(branch) => branch,
    };
    Msg::RepositoryInfoFetched(Ok(branch))
}

async fn send_message() -> Msg {
    let message = SendMessageRequestBody {
        name: "Mark Watney".into(),
        email: "mark@crypt.kk".into(),
        message: "I wanna be like Iron Man".into(),
    };

    let request = Request::new(CONTACT_URL)
        .method(Method::Post)
        .json(&message)
        .expect("Serialization failed");

    // TODO chain?
    let response = match fetch(request).await {
        Err(err) => return Msg::MessageSent(Err(err)),
        Ok(response) => response,
    };
    let response_message = match response.json().await {
        Err(err) => return Msg::MessageSent(Err(err)),
        Ok(response_message) => response_message,
    };
    Msg::MessageSent(Ok(response_message))
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        md!["# Repo info"],
        div![format!(
            "Name: {}, SHA: {}",
            model.branch.name, model.branch.commit.sha
        )],
        raw!["<hr>"],
        button![
            ev(Ev::Click, |_| Msg::SendMessage),
            "Send an urgent message (see console log)"
        ],
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
