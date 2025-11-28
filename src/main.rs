use rocket::fs::FileServer;
use rocket::response::Responder;
use rocket::serde::json::{Json, Value, serde_json::json, serde_json::to_value};
use rocket_dyn_templates::{Template, context};
use std::io::Cursor;
use std::io::{BufRead, BufReader};
use std::process::Command;

use std::collections::HashMap;
use std::str::FromStr;

#[macro_use]
extern crate rocket;

#[derive(Responder)]
enum SlurmWebResponse {
    #[response(status = 200)]
    HTML(Template),
    #[response(status = 200)]
    JSON(Json<Value>),
}

fn squeue(jobname: Option<String>) -> Option<Value> {
    let base_args = vec!["-a", "-o", "%all"];
    let args: Vec<&str> = base_args
        .into_iter()
        .chain(
            jobname
                .as_ref()
                .map(|name| vec!["-n", name.as_str()])
                .unwrap_or_default(),
        )
        .collect();

    if let Some(output) = Command::new("squeue").args(&args).output().ok() {
        if output.status.success() {
            // Read stdout as a file, line by line, parsing |-separated text
            // First line is the header
            let mut res: HashMap<String, HashMap<String, String>> = HashMap::new();
            let mut lines = BufReader::new(Cursor::new(output.stdout)).lines();
            let header = lines.next()?.ok()?;
            let fields = header.split('|').collect::<Vec<&str>>();
            for line in lines {
                let line = line.ok()?;
                let values = line.split('|').collect::<Vec<&str>>();
                let mut job: HashMap<String, String> = HashMap::new();
                for (i, field) in fields.iter().enumerate() {
                    if let Some(value) = values.get(i) {
                        // Ignore field with no name
                        if !field.to_string().is_empty() {
                            job.insert(field.to_string(), value.to_string());
                        }
                    }
                }
                res.insert(
                    job.get("JOBID")
                        .map_or("UNKNOWN".to_string(), |v| v.clone()),
                    job,
                );
            }
            Some(to_value(res).unwrap())
        } else {
            None
        }
    } else {
        // Mock data
        Value::from_str(
            r#"{
          "4725": {
            "ACCOUNT": "urgent",
            "ARRAY_JOB_ID": "4725",
            "ARRAY_TASK_ID": "N/A",
            "COMMAND": "(null)",
            "COMMENT": "(null)",
            "CONTIGUOUS": "0",
            "CORES_PER_SOCKET": "*",
            "CORE_SPEC": "N/A",
            "CPUS": "28",
            "DEPENDENCY": "(null)",
            "END_TIME": "NONE",
            "EXC_NODES": "",
            "EXEC_HOST": "slurm-gpu04",
            "FEATURES": "(null)",
            "GROUP": "1004",
            "JOBID": "4725",
            "LICENSES": "(null)",
            "MIN_CPUS": "28",
            "MIN_MEMORY": "256G",
            "MIN_TMP_DISK": "0",
            "NAME": "wrap",
            "NICE": "0",
            "NODELIST": "slurm-gpu04",
            "NODELIST(REASON)": "slurm-gpu04",
            "NODES": "1",
            "OVER_SUBSCRIBE": "OK",
            "PARTITION": "GENOMIQUE-CPU",
            "PRIORITY": "4294897553",
            "QOS": "urgent",
            "REASON": "None",
            "REQ_NODES": "",
            "RESERVATION": "(null)",
            "S:C:T": "*:*:*",
            "SCHEDNODES": "(null)",
            "SOCKETS_PER_NODE": "*",
            "ST": "R",
            "START_TIME": "2025-11-28T10:20:05",
            "STATE": "RUNNING",
            "SUBMIT_TIME": "2025-11-28T10:20:04",
            "THREADS_PER_CORE": "*",
            "TIME": "34:07",
            "TIME_LEFT": "UNLIMITED",
            "TIME_LIMIT": "UNLIMITED",
            "TRES_PER_NODE": "gres:gpu:a100:2",
            "UID": "1001",
            "USER": "charles",
            "WCKEY": "(null)",
            "WORK_DIR": "/some/work/directory"
          }
        }"#,
        )
        .ok()
    }
}

#[get("/squeue?<format>&<jobname>")]
fn squeue_get(format: Option<String>, jobname: Option<String>) -> SlurmWebResponse {
    let format = format.unwrap_or("json".to_string());

    if let Some(json) = squeue(jobname) {
        match format.as_str() {
            "json" => SlurmWebResponse::JSON(Json(json)),
            "html" => SlurmWebResponse::HTML(Template::render("squeue", context! {})),
            _ => SlurmWebResponse::JSON(Json(json!({"error": "unknown format"}))),
        }
    } else {
        match format.as_str() {
            "json" => SlurmWebResponse::JSON(Json(json!({"error": "squeue failed"}))),
            "html" => SlurmWebResponse::HTML(Template::render("error", context! {})),
            _ => SlurmWebResponse::JSON(Json(json!({"error": "unknown format"}))),
        }
    }
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from("./static"))
        .mount("/", routes![squeue_get])
        .attach(Template::fairing())
}
