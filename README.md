# Gitlab Helper
A command-line utility for interacting with Gitlab from a course organization perspective

## Brightspace Integration
Currently this requires cookies from brightspace to authenticate, you need the two cookies called: `d2lSessionVal` and `d2lSecureSessionVal`.
Put these into the correct env var.

Sometime you also need your brightspace (lti) session id, this can be found in the cookies of <https://group-impexp.lti.tudelft.nl/>


## Note on group creation
For GitLab group creation there now also exists GitBull: <https://gitbull.ewi.tudelft.nl/>,
this might be a better fit for you. Some commands in the program can generate CSVs that are compatible with gitbull using the `--gitbull` CLI option,
e.g. downloading the classlist from Brightspace.
