use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use gitlab::Gitlab;

mod create_repos;
mod models;
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

    /// Create Gitlab repos for individual students under a certain group and
    CreateIndividualRepos {
        /// Parent Group ID
        #[arg(required = true)]
        group_id: u64,

        /// Template Repository to Create per Student
        #[arg(required = true)]
        template_repository: String,

        /// Brightspace student list (see README)
        #[arg(required = true)]
        student_list: PathBuf,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client = Gitlab::new(args.host, args.token).unwrap();

    match args.cmd {
        SubCommand::Projects { group_id } => projects::list(&client, group_id)?,
        SubCommand::Unprotect { group_id, branch } => {
            projects::unprotect(&client, group_id, &branch)?;
        }
        SubCommand::CreateIndividualRepos {
            group_id,
            template_repository,
            student_list,
        } => {

        },
    }

    Ok(())
}
