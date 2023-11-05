use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};
use std::sync::{Arc, Mutex};

pub struct SharedState {
    files_read: bool,
    contents: Vec<String>,
}

pub async fn read_naughty_words(file_paths: Vec<&'static str>) -> io::Result<Vec<String>> {

    let state = Arc::new(Mutex::new(SharedState { 
        files_read: false,
        contents: Vec::new(),
    }));

    let state_clone = Arc::clone(&state);
    let file_paths_clone = file_paths.clone();

    // Spawn the async task
    tokio::spawn(async move {
        for path in file_paths_clone {
            // Perform the file read operation
            let mut file = File::open(path).await.unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).await.unwrap();

            // Split the contents into lines, filter out lines starting with '#' or empty, and collect the rest
            let mut filtered_contents: Vec<String> = contents.lines()
            .filter(|line| !line.trim_start().starts_with('#') && !line.is_empty())
            .map(|line| line.to_string())
            .collect();

            // If you need the filtered contents as a single String, you can rejoin them
            //let filtered_string = filtered_contents.join("\n");

            // Acquire the lock and push the contents into the vector
            let mut state = state_clone.lock().unwrap();
            state.contents.append(&mut filtered_contents);
        }

        // Set the flag after reading all files
        let mut state = state_clone.lock().unwrap();
        state.files_read = true;
    });

    // Wait for the task to complete
    loop {
        // Acquire the lock and check the flag
        let state = state.lock().unwrap();
        if state.files_read {
            return Ok(state.contents.clone());
        }
    }
}
