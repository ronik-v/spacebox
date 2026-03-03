pub const TTL_MINUTES: i32 = 15;

pub const EMAIL_SUBJECT_TEMPLATE: &str = "Account confirmation — {{app_name}}";

pub const EMAIL_HTML_BODY_TEMPLATE: &str = r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <title>Confirmation — {{app_name}}</title>
    <style>
      body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial; background:#f6f6f6; margin:0; padding:0; }
      .container { max-width:600px; margin:40px auto; background:#ffffff; padding:24px; border-radius:8px; box-shadow:0 2px 6px rgba(0,0,0,0.08); }
      h2 { margin-top:0; }
      .code { display:block; font-size:28px; letter-spacing:4px; font-weight:700; text-align:center; background:#f0f0f0; padding:12px 16px; border-radius:6px; margin:20px 0; }
      .small { color:#777; font-size:13px; }
      hr { border:none; border-top:1px solid #eee; margin:20px 0; }
      .footer { color:#999; font-size:12px; }
    </style>
  </head>
  <body>
    <div class="container">
      <h2>Welcome to {{app_name}}!</h2>
      <p>To complete your registration, enter the following confirmation code:</p>

      <div class="code">{{code}}</div>

      <p class="small">The code is valid for {{ttl_minutes}} minutes. If you did not request this, please ignore this email.</p>

      <hr />

      <p class="footer">If the email is not displayed correctly, enter the code manually. &copy; {{year}} {{app_name}}</p>
    </div>
  </body>
</html>
"#;
