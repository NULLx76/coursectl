# Course Helper (coursectl)
A command-line utility for interacting with Gitlab and Brightspace to automate some course management functions

You can run `cargo run -- help` to view help information.

## Brightspace Integration
Currently this requires cookies from brightspace to authenticate, you need the two cookies called: `d2lSessionVal` and `d2lSecureSessionVal`.
Sometime you also need your brightspace (lti) session id, this can be found in the cookies of <https://group-impexp.lti.tudelft.nl/>

The program will automatically fetch these cookies from either Firefox or Chromium's cookies database.
If you're getting unauthorized errors please visit these websites in your browser or manually set the cookies.

