#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;
use rocket::*;

use serde_derive::*;

use anyhow::Result;

use std::collections::BTreeMap;

#[derive(Debug, Default, FromForm, Serialize, Deserialize)]
struct Entry<N, D> {
	finished: bool,
	name: N,
	description: D,
}

#[get("/")]
fn render(cookies: Cookies) -> Result<Html<String>> {
	let mut total = 0;
	let mut todo = 0;

	#[derive(Default)]
	struct Todos<'a> {
		entries: BTreeMap<usize, Entry<&'a str, &'a str>>,
	}

	const _: () = {
		use core::fmt::*;

		impl<'a> Display for Todos<'a> {
			fn fmt(&self, f: &mut Formatter) -> Result {
				for (id, entry) in self.entries.iter() {
					write!(
						f,
						r#"
<li>
	<form
		method="POST"
		action="\update\{id}"
		id="update_{id}"
	>
			<input type="checkbox" name="entry_finished" value="{}">
			<input required
				name="entry_name"
				form="{id}"
				placeholder="Name"
				value="{}"
			>
			<input
				name="entry_description"
				form="{id}"
				placeholder="Description"
				value="{}"
				size="48"
			>
			<input type="radio" name="command" value ="Delete"> Delete
			<input type="radio" name="command" value ="Update"> Update
			<input name="commit" form="update_{id}" type="submit" value="Commit">
		<form>
 </li><br>"#,
						entry.finished,
						entry.name,
						entry.description,
						id = id,
					)?
				}
				Ok(())
			}
		}
	};

	let mut todos = Todos::default();

	for cookie in cookies.iter() {
		let entry = serde_json::from_str::<Entry<&str, &str>>(cookie.value())?;
		let id: usize = cookie.name().parse()?;

		total += 1;
		todo += entry.finished as usize;

		todos.entries.insert(id, entry);
	}

	Ok(Html(format!(
		r#"<!DOCTYPE html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<title>Todo | [{} / {}]</title>
	</head>
	<body>
		<h1>Todo</h1>
		<ol>
			{}
		</ol>
		<form
			method="POST"
			action="\new_entry"
			id="new_entry"
		>
			<input type="submit" value="New entry">
		</form>
	</body>
</html>"#,
		todo, total, todos,
	)))
}

#[post("/new_entry")]
fn new_entry(mut cookies: Cookies) -> Result<Redirect> {
	let id = cookies.iter().count();

	cookies.add(Cookie::new(
		id.to_string(),
		serde_json::to_string(&Entry::<&str, &str>::default())?,
	));

	Ok(Redirect::to(uri![render]))
}

#[derive(Debug, FromFormValue)]
enum Command {
	Update,
	Delete,
}

#[derive(Debug, FromForm)]
struct Update<N, D> {
	finished: bool,
	name: N,
	description: D,
	command: Command,
}

#[post("/update/<id>", data = "<update>")]
fn update(id: usize, update: Form<Update<String, String>>, cookies: Cookies) {}

fn main() {
	ignite()
		.mount("/", routes![render, new_entry, update])
		.launch();
}
