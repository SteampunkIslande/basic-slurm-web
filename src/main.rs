use rocket::fs::FileServer;
use rocket::response::Responder;
use rocket::serde::json::{Json, Value, serde_json::json, serde_json::to_value};
use rocket_dyn_templates::{Template, context};
use std::io::Cursor;
use std::io::{BufRead, BufReader};
use std::process::Command;

use std::collections::HashMap;

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
                        job.insert(field.to_string(), value.to_string());
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
        Some(json!({
            "1234": {"JOBID": "1234", "JOBNAME": "ml_training", "USER": "alice", "PARTITION": "gpu", "STATE": "RUNNING", "TIME": "2:15:30", "NODES": "1", "NODELIST": "gpu-node01", "CPUS": "8", "MEM": "32G", "TIMELIMIT": "24:00:00", "COMMENT": "Training ResNet model on ImageNet"},
            "5678": {"JOBID": "5678", "JOBNAME": "data_processing", "USER": "bob", "PARTITION": "cpu", "STATE": "PENDING", "TIME": "0:00:00", "NODES": "2", "NODELIST": "(null)", "CPUS": "16", "MEM": "64G", "TIMELIMIT": "12:00:00", "COMMENT": "Waiting for resources"},
            "9012": {"JOBID": "9012", "JOBNAME": "simulation", "USER": "charlie", "PARTITION": "hpc", "STATE": "COMPLETED", "TIME": "8:45:12", "NODES": "4", "NODELIST": "hpc-[001-004]", "CPUS": "128", "MEM": "256G", "TIMELIMIT": "10:00:00", "COMMENT": "Molecular dynamics simulation finished successfully"},
            "3456": {"JOBID": "3456", "JOBNAME": "failed_job", "USER": "david", "PARTITION": "debug", "STATE": "FAILED", "TIME": "0:05:23", "NODES": "1", "NODELIST": "debug-node01", "CPUS": "2", "MEM": "4G", "TIMELIMIT": "0:30:00", "COMMENT": "Out of memory error"},
            "7890": {"JOBID": "7890", "JOBNAME": "big_computation", "USER": "eve", "PARTITION": "cpu", "STATE": "CANCELLED", "TIME": "1:30:45", "NODES": "8", "NODELIST": "cpu-[010-017]", "CPUS": "256", "MEM": "512G", "TIMELIMIT": "48:00:00", "COMMENT": "Cancelled by user request"},
            "1111": {"JOBID": "1111", "JOBNAME": "test_script", "USER": "alice", "PARTITION": "debug", "STATE": "TIMEOUT", "TIME": "0:30:00", "NODES": "1", "NODELIST": "debug-node02", "CPUS": "1", "MEM": "2G", "TIMELIMIT": "0:30:00", "COMMENT": "Job exceeded time limit"},
            "2222": {"JOBID": "2222", "JOBNAME": "array_job", "USER": "bob", "PARTITION": "gpu", "STATE": "RUNNING", "TIME": "1:22:18", "NODES": "2", "NODELIST": "gpu-[node02-node03]", "CPUS": "16", "MEM": "128G", "TIMELIMIT": "6:00:00", "COMMENT": "Parameter sweep in progress"},
            "3333": {"JOBID": "3333", "JOBNAME": "backup_task", "USER": "system", "PARTITION": "maintenance", "STATE": "SUSPENDED", "TIME": "0:12:05", "NODES": "1", "NODELIST": "storage-node01", "CPUS": "4", "MEM": "8G", "TIMELIMIT": "2:00:00", "COMMENT": "Suspended for maintenance window"},
            "4444": {"JOBID": "4444", "JOBNAME": "neural_net", "USER": "charlie", "PARTITION": "gpu", "STATE": "PENDING", "TIME": "0:00:00", "NODES": "1", "NODELIST": "(null)", "CPUS": "32", "MEM": "128G", "TIMELIMIT": "72:00:00", "COMMENT": "Priority: high priority job"},
            "5555": {"JOBID": "5555", "JOBNAME": "preprocessing", "USER": "eve", "PARTITION": "cpu", "STATE": "CONFIGURING", "TIME": "0:00:00", "NODES": "1", "NODELIST": "cpu-008", "CPUS": "8", "MEM": "16G", "TIMELIMIT": "4:00:00", "COMMENT": "Setting up environment"}
        }))
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
