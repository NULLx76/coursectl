use std::{collections::HashMap, fs::File, path::PathBuf};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, ContextCompat, Result};
use gitlab::{AccessLevel, Gitlab};
use itertools::Itertools;
use models::StudentGroupEntry;

use crate::models::Group;

mod brightspace;
mod create_repos;
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
        #[arg(short,long, default_value_t = AccessLevel::Developer.into())]
        access_level: u64,
    },

    /// Create Gitlab groups based on a CSV file, see `group_example.csv` for the format
    CreateGroupsFromCsv {
        /// Parent Group ID
        #[arg(short, long, required = true)]
        group_id: u64,

        /// Path to CSV file
        #[arg(short, long, required = true)]
        csv: PathBuf,

        /// Template Repository to use for each student (has to be public)
        #[arg(short, long = "template", required = true)]
        template_repository: String,

        /// Brightspace Organizational Unit (ID)
        #[arg(long = "ou", required = true)]
        brightspace_ou: u64,

        /// Specify the accesslevel of the users to be added to the repo
        ///
        /// Anonymous => 0,  
        /// Guest => 10,  
        /// Reporter => 20,  
        /// Developer => 30,  
        /// Maintainer => 40,
        /// Owner => 50,
        /// Admin => 60,
        #[arg(short,long, default_value_t = AccessLevel::Developer.into())]
        access_level: u64,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client =
        Gitlab::new(&args.host, &args.gitlab_token).wrap_err("failed to create gitlab client")?;

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
            if template_repository.contains(&args.host) {
                let (proto, suff) = template_repository
                    .split_once("://")
                    .wrap_err("invalid template url")?;
                template_repository = format!(
                    "{proto}://{}:{}@{suff}",
                    &args.gitlab_user, &args.gitlab_token
                );
            }

            create_repos::create_individual_repos(
                &client,
                &repo_name_prefix,
                group_id,
                &template_repository,
                access_level.into(),
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
                println!("{:07}, {}", entry.student_number.unwrap_or(0), entry.netid)
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

        SubCommand::CreateGroupsFromCsv {
            group_id,
            csv,
            template_repository,
            brightspace_ou,
            access_level,
        } => {
            let brightspace_students = brightspace::get_students(
                &args.brightspace_base_url,
                &args.brightspace_cookie,
                brightspace_ou,
            )?;

            let mut rdr = csv::Reader::from_reader(
                File::open(csv.as_path())
                    .wrap_err_with(|| format!("Could not open file {:?}", csv))?,
            );
            let mut groups: HashMap<String, Group> = HashMap::new();
            for row in rdr.deserialize() {
                let group_entry: StudentGroupEntry = row?;

                // yes I know this is O(bad)
                let student = brightspace_students
                    .iter()
                    .find(|b| b.netid == group_entry.netid.to_lowercase().trim())
                    .wrap_err_with(|| {
                        format!("student not found in brightspace {:?}", group_entry)
                    })?;

                groups
                    .entry(group_entry.group_name.clone())
                    .and_modify(|f| {
                        f.members.push(student.clone());
                    })
                    .or_insert(Group {
                        name: group_entry.group_name.clone(),
                        members: vec![student.clone()],
                    });
            }

            create_repos::create_group_repos(
                &client,
                group_id,
                &template_repository,
                AccessLevel::from(access_level),
                groups.values().cloned().collect_vec(),
                args.dry_run,
            )?;
        }
    }

    Ok(())
}
