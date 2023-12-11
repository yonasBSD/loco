use std::{collections::HashMap, env::current_dir};

use chrono::Utc;
use duct::cmd;
use lazy_static::lazy_static;
use rrgen::RRgen;
use serde_json::json;

use crate::app::Hooks;

const MODEL_T: &str = include_str!("templates/model.t");
const MODEL_TEST_T: &str = include_str!("templates/model_test.t");

use super::collect_messages;
use crate::{errors::Error, Result};

lazy_static! {
    static ref TYPEMAP: HashMap<&'static str, &'static str> = HashMap::from([
        ("text", "text"),
        ("string", "string_null"),
        ("string!", "string"),
        ("string^", "string_uniq"),
        ("int", "integer_null"),
        ("int!", "integer"),
        ("int^", "integer_uniq"),
        ("bool", "bool_null"),
        ("bool!", "bool"),
        ("ts", "timestamp_null"),
        ("ts!", "timestamp"),
        ("uuid", "uuid"),
    ]);
}

pub fn generate<H: Hooks>(
    rrgen: &RRgen,
    name: &str,
    fields: &[(String, String)],
) -> Result<String> {
    let pkg_name: &str = H::app_name();
    let ts = Utc::now();

    let mut columns = Vec::new();
    let mut references = Vec::new();
    for (fname, ftype) in fields {
        if ftype == "references" {
            let fkey = format!("{fname}_id");
            columns.push((fkey.clone(), "integer"));
            // user, user_id
            references.push((fname, fkey));
        } else {
            let schema_type = TYPEMAP.get(ftype.as_str()).ok_or_else(|| {
                Error::Message(format!(
                    "type: {} not found. try any of: {:?}",
                    ftype,
                    TYPEMAP.keys()
                ))
            })?;
            columns.push((fname.to_string(), *schema_type));
        }
    }

    let vars = json!({"name": name, "ts": ts, "pkg_name": pkg_name, "columns": columns, "references": references});
    let res1 = rrgen.generate(MODEL_T, &vars)?;
    let res2 = rrgen.generate(MODEL_TEST_T, &vars)?;

    let cwd = current_dir()?;
    let _ = cmd!("cargo", "run", "--", "db", "migrate",)
        .stderr_to_stdout()
        .dir(cwd.as_path())
        .run()?;
    let _ = cmd!("cargo", "run", "--", "db", "entities",)
        .stderr_to_stdout()
        .dir(cwd.as_path())
        .run()?;

    let messages = collect_messages(vec![res1, res2]);
    Ok(messages)
}
