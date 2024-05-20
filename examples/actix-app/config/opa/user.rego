package app.user

import rego.v1

default allow := false

allow if user_is_admin

allow if {
    input.method == "GET"
    input.path == ["user", input.session.user_id, "view"]
}

user_is_admin if "admin" in input.session.roles
