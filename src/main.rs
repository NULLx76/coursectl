use clap::{Parser, Subcommand};
use gitlab::Gitlab;

mod projects;

#[derive(Parser, Debug)]
struct Args {
    /// Gitlab host
    #[arg(long, default_value = "gitlab.ewi.tudelft.nl")]
    host: String,

    /// Gitlab API token
    #[arg(long, env = "GITLAB_TOKEN", hide_env_values = true)]
    token: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Lists all projects in a given group with ssh clone url
    Projects {
        /// Gitlab group ID
        #[arg(required = true)]
        group_id: u64,
    },

    /// Unprotects a given branch on all projects in a given group
    Unprotect {
        /// Gitlab group ID
        #[arg(required = true)]
        group_id: u64,

        /// Branch to unprotect
        #[arg(required = true)]
        branch: String,
    },
}

fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client = Gitlab::new(args.host, args.token).unwrap();

    match args.cmd {
        SubCommand::Projects { group_id} => projects::list(&client, group_id),
        SubCommand::Unprotect { group_id, branch } => {
            projects::unprotect(&client, group_id, &branch);
        }
    }
}
