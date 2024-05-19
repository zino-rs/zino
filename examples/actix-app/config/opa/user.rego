package app.user

import rego.v1

default allow := false

allow if {
    input.method == "GET"
    input.path == ["user", input.user_session.user_id, "view"]
}
