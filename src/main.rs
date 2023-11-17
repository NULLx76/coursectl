use std::{fs::File, path::PathBuf};

use crate::brightspace::get_groups;
use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use gitlab::{api::common::AccessLevel, Gitlab};

use crate::git::create_repos;
use crate::git::create_repos::create_group_repos;

mod brightspace;
mod git;
mod models;
mod projects;

#[derive(Parser, Debug)]
struct Args {
    /// Gitlab host
    #[arg(long, default_value = "gitlab.ewi.tudelft.nl")]
    host: String,

    /// Gitlab API token
    #[arg(long = "token", env = "GITLAB_TOKEN", hide_env_values = true)]
    gitlab_token: String,

    /// Gitlab API user (connected to the token)
    #[arg(long = "user", env = "GITLAB_USER", hide_env_values = true)]
    gitlab_user: String,

    #[arg(long, default_value = "https://brightspace.tudelft.nl")]
    brightspace_base_url: http::Uri,

    #[arg(long, env = "BRIGHTSPACE_COOKIE", hide_env_values = true)]
    brightspace_cookie: String,

    #[arg(long, env = "BRIGHTSPACE_SESSIONID", hide_env_values = true)]
    brightspace_session_id: String,

    #[arg(long, default_value_t = false)]
    dry_run: bool,

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

    ClasslistCsv {
        #[arg(required = true)]
        course_id: u64,

        #[arg(short, long = "file", default_value = "classlist.csv")]
        output_file: PathBuf,
    },

    /// Unprotects a given branch on all projects in a given group
    Unprotect {
        /// Gitlab group ID
        #[arg(required = true)]
        group_id: u64,

        /// Branch to unprotect
        #[arg(default_value = "main")]
        branch: String,
    },

    Unfork {
        #[arg(required = true)]
        group_id: u64
    },

    GetClassList {
        #[arg(required = true)]
        course_id: u64,
    },

    /// Create Gitlab repos for individual students under a certain group and
    CreateIndividualRepos {
        /// Parent Group ID
        #[arg(short, long, required = true)]
        group_id: u64,

        /// Template Repository to use for each student (has to be public)
        #[arg(short, long = "template", required = true)]
        template_repository: String,

        /// Brightspace Organizational Unit (ID)
        #[arg(long = "ou", required = true)]
        brightspace_ou: u64,

        /// Prefix to add to all created repositories
        #[arg(short = 'p', long = "prefix")]
        repo_name_prefix: Option<String>,

        /// Specify the accesslevel of the users to be added to the repo
        ///
        /// Anonymous => 0,  
        /// Guest => 10,  
        /// Reporter => 20,  
        /// Developer => 30,  
        /// Maintainer => 40,
        /// Owner => 50,
        /// Admin => 60,
        #[arg(short, long, default_value_t = AccessLevel::Developer.as_u64())]
        access_level: u64,
    },
    CreateGroupsFromBrightspace {
        /// Gitlab parent Group ID
        #[arg(short, long, required = true)]
        group_id: u64,

        /// The brightspace groups category id
        #[arg(short, long = "group_id", required = true)]
        brightspace_group_category_id: u64,

        /// Template Repository to use for each student
        #[arg(short, long = "template", required = true)]
        template_repository: String,

        /// Specify the accesslevel of the users to be added to the repo
        ///
        /// Anonymous => 0,  
        /// Guest => 10,  
        /// Reporter => 20,  
        /// Developer => 30,  
        /// Maintainer => 40,
        /// Owner => 50,
        /// Admin => 60,
        #[arg(short, long, default_value_t = AccessLevel::Developer.as_u64())]
        access_level: u64,
    },
}

fn u64_to_access_level(access: u64) -> AccessLevel {
    if access >= 60 {
        AccessLevel::Admin
    } else if access >= 50 {
        AccessLevel::Owner
    } else if access >= 40 {
        AccessLevel::Maintainer
    } else if access >= 30 {
        AccessLevel::Developer
    } else if access >= 20 {
        AccessLevel::Reporter
    } else if access >= 10 {
        AccessLevel::Guest
    } else {
        AccessLevel::Anonymous
    }
}

fn authenticate_template_repo_url(
    mut template_repository: String,
    host: &str,
    user: &str,
    token: &str,
) -> Result<String> {
    if template_repository.contains(host) {
        let (proto, suff) = template_repository
            .split_once("://")
            .wrap_err("invalid template url")?;
        template_repository = format!("{proto}://{user}:{token}@{suff}");
    }

    Ok(template_repository)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client =
        Gitlab::new(&args.host, &args.gitlab_token).wrap_err("failed to create git client")?;

    match args.cmd {
        SubCommand::Projects { group_id } => projects::list(&client, group_id)?,
        SubCommand::Unprotect { group_id, branch } => {
            projects::unprotect(&client, group_id, &branch, args.dry_run)?;
        }
        SubCommand::CreateIndividualRepos {
            group_id,
            mut template_repository,
            repo_name_prefix,
            access_level,
            brightspace_ou,
        } => {
            template_repository = authenticate_template_repo_url(
                template_repository,
                &args.host,
                &args.gitlab_user,
                &args.gitlab_token,
            )?;

            create_repos::create_individual_repos(
                &client,
                &repo_name_prefix,
                group_id,
                &template_repository,
                u64_to_access_level(access_level),
                &args.brightspace_cookie,
                &args.brightspace_base_url,
                brightspace_ou,
                args.dry_run,
            )?;
        }
        SubCommand::GetClassList { course_id } => {
            let mut list = brightspace::get_students(
                &args.brightspace_base_url,
                &args.brightspace_cookie,
                course_id,
            )?;
            list.sort_by_key(|s| s.netid.clone());
            for entry in list {
                println!("{:07}, {}", entry.student_number.unwrap_or(0), entry.netid);
            }
        }
        SubCommand::ClasslistCsv {
            course_id,
            output_file,
        } => {
            let f = File::create(output_file).wrap_err("could not create output file")?;
            let mut wtr = csv::Writer::from_writer(f);

            brightspace::get_students(
                &args.brightspace_base_url,
                &args.brightspace_cookie,
                course_id,
            )?
            .iter()
            .try_for_each(|el| wtr.serialize(el))?;

            wtr.flush()?;
        }
        SubCommand::CreateGroupsFromBrightspace {
            group_id,
            brightspace_group_category_id,
            mut template_repository,
            access_level,
        } => {
            if args.brightspace_session_id.is_empty() {
                return Err(eyre!("brightspace_session_id missing!"));
            }

            template_repository = authenticate_template_repo_url(
                template_repository,
                &args.host,
                &args.gitlab_user,
                &args.gitlab_token,
            )?;

            let groups = get_groups(
                &args.brightspace_session_id,
                &brightspace_group_category_id.to_string(),
            )?;

            create_group_repos(
                &client,
                group_id,
                &template_repository,
                u64_to_access_level(access_level),
                &groups,
                args.dry_run,
            )?;
        },

        SubCommand::Unfork { group_id } => {
            projects::unfork(&client, group_id, args.dry_run)?;
        }
    }

    Ok(())
}
