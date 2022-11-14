use clap::{Parser, Subcommand};
use dotenv;
use gitlab::Gitlab;

mod projects;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "gitlab.ewi.tudelft.nl")]
    host: String,

    #[arg(long, env = "GITLAB_TOKEN", hide_env_values = true)]
    token: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Projects {
        #[arg(required = true)]
        id: u64,
    },
}

fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client = Gitlab::new(args.host, args.token).unwrap();

    match args.cmd {
        SubCommand::Projects { id } => projects::list_projects(&client, id),
    }
}
