pub mod requests_to_server;


/*
// for TemplateRequestCategory:
template_request_category! {
    exported_macro_name: $GENERATED_MACRO_NAME,
    template: {
        category_name: "$CATEGORY_NAME",
        base_path: "$BASE_PATH",
        read: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING, "optional docs",
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
            },
        },
        create: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING, "optional docs",
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
            },
        },
        edit: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING, "optional docs",
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
            },
        },
        delete: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING, "optional docs",
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
            },
        },
        patch: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING, "optional docs",
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE, "optional docs",
                    ...
                ],
            },
        }
    }
}
 */