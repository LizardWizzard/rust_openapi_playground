## Play with two openapi frameworks poem and utoipa + axum 

### try_poem contains poem code

pros:
- strictly typed, spec is inferred from handler parameter/return type, so they cannot diverge
- no duplication of paths compared to utoipa
cons:
- harder to deal with common errors across handlers because each response type is unique. (So each time you get e g tenant missing from internal api you have to convert it to MyEndpointTenantIsMissing). This probably can be worked around with macro, but still.
- poem is separate framework
- not easy to introduce new "atomic" types to spec because they have to implement a bunch of traits. See try_utoipa/src/lib.rs for example with tenant id

### try_utoipa contains utoipa + axum code

pros:
- utoipa is framework agnostic
- combination with axum which is quite popular and belongs to tokio org

cons:
- no go to definition in utoipa macro annotations (the whole idea os custom macro dsl)
- you have to fill path for a route in two different places, it wont save you from a typo
- Macro input is not checked, you can reference non existing type in spec. It wont give you compilation error
- Handler returns impl IntoResponse, so it can be anything that not nessesarily have to match the spec


