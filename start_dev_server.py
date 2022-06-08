from datetime import timedelta
from prefect import Flow, task
import subprocess


@task(retry_delay=timedelta(seconds=3), max_retries=10, log_stdout=True)
def start_actix_web_server():
    return_code = subprocess.call("cargo run", shell=True)
    assert return_code != 3, "Rust process aborted!"


def main():
    with Flow("Flow: Start Actix Web Server in Dev") as flow:
        start_actix_web_server()

    flow.run()


if __name__ == "__main__":
    main()
