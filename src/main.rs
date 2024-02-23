use std::{cell::Cell, rc::Rc, path::Path};
use std::process::{Command, Stdio};
use appindicator3::{prelude::*, IndicatorStatus};
use appindicator3::{Indicator, IndicatorCategory};
use gtk::prelude::*;

fn check_running() -> bool {
    let status = Command::new("microk8s.status")
        .arg("|")
        .arg("grep")
        .arg("-v")
        .arg("not")
        .status()
        .expect("failed to get microk8s status");
    !status.success()
}

fn start_dss() -> bool {
    let status = Command::new("microk8s.start")
        .status()
        .expect("failed to start microk8s");
    status.success()
}

fn stop_dss() -> bool {
    let status = Command::new("microk8s.stop")
        .status()
        .expect("failed to stop microk8s");
    status.success()
}

fn get_mlflow_url() -> String {
    let output = Command::new("microk8s.kubectl")
        .arg("get")
        .arg("svc")
        .arg("-n")
        .arg("dss")
        .arg("-o")
        .arg("jsonpath=\"{.spec.clusterIP}\"")
        .arg("mlflow")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to get mlflow URL");
    let ip = String::from_utf8(output.stdout).unwrap().trim_end_matches('"').trim_start_matches('"').to_owned();
    let url = format!("http://{ip}");
    println!("MLFlow {}", url);
    url
}

fn get_notebook_url() -> String {
    let output = Command::new("microk8s.kubectl")
        .arg("get")
        .arg("svc")
        .arg("-n")
        .arg("dss")
        .arg("-o")
        .arg("jsonpath=\"{.spec.clusterIP}\"")
        .arg("user-notebook-tensorflow")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to get user-notebook-tensorflow URL");
    let ip = String::from_utf8(output.stdout).unwrap().trim_end_matches('"').trim_start_matches('"').to_owned();
    let url = format!("http://{ip}");
    println!("Notebook {}", url);
    url
}

fn main() {
    gtk::init().unwrap();

    let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/icons");
    let running = Rc::new(Cell::new(check_running()));

    let menu = gtk::Menu::new();

    let indicator = Indicator::builder("DSS Indicator")
        .category(IndicatorCategory::ApplicationStatus)
        .menu(&menu)
        .icon_theme_path(icon_path.to_str().unwrap())
        .icon("dss-status", "dss-status")
        .status(IndicatorStatus::Active)
        .build();
    indicator.set_attention_icon_full("dss-running", "dss-running");

    let start_stop_item = gtk::MenuItem::with_label("Start");
    start_stop_item.connect_activate(glib::clone!(@weak indicator, @weak running => move |item| {
        if !running.get() {
            indicator.set_icon_full("dss-status", "dss-status");
            if start_dss() {
                running.set(true);
                indicator.set_icon_full("dss-running", "dss-running");
                item.set_label("Stop");
            }
        } else {
            indicator.set_icon_full("dss-status", "dss-status");
            if stop_dss() {
                running.set(false);
                indicator.set_icon_full("dss-stopped", "dss-stopped");
                item.set_label("Start");
            }
        }
    }));

    let mlflow_item = gtk::MenuItem::with_label("MLFlow");
    mlflow_item.connect_activate(|item| {
        let mlflow_url = get_mlflow_url();
        println!("MLFlow {}", mlflow_url);
        open::that(mlflow_url);
    });

    let notebook_item = gtk::MenuItem::with_label("Notebook");
    notebook_item.connect_activate(|item| {
        let notebook_url = get_notebook_url();
        println!("Notebook {}", notebook_url);
        open::that(notebook_url);
    });

    menu.append(&start_stop_item);
    start_stop_item.show_all();

    menu.append(&mlflow_item);
    mlflow_item.show_all();

    menu.append(&notebook_item);
    notebook_item.show_all();
    
    start_stop_item.connect_label_notify(glib::clone!(@weak running, @weak mlflow_item, @weak notebook_item => move |item|{
        println!("label changed");
        if running.get() {
            mlflow_item.show_all();
            notebook_item.show_all();
        } else {
            mlflow_item.hide();
            notebook_item.hide();
        }
    }));

    if running.get() {
        indicator.set_icon_full("dss-running", "dss-running");
        start_stop_item.set_label("Stop");
    } else {
        indicator.set_icon_full("dss-stopped", "dss-stopped");
        start_stop_item.set_label("Start");
    }

    gtk::main();
}
