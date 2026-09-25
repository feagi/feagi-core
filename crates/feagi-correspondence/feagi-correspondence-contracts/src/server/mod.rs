pub mod requests_to_server;

/*

for CompleteRequestResponsesTemplate:
{
    exported_macro_name: $GENERATED_MACRO_NAME,
    template: {
        root_path: "$ROOT_PATH",
        categories: [
            macro_category_a!(),
            macro_category_b!(),
            macro_category_c!(),
        ]
    }
}
 */



/*
for TemplateRequestCategory:
{
    exported_macro_name: $GENERATED_MACRO_NAME,
    template: {
        category_name: "$CATEGORY_NAME",
        read: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
            },
        },
        create: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
            },
        },
        edit: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
            },
        },
        delete: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE,
                    ...
                ],
            },
        },
        patch: {
            "$PATH_EXTENSION": {
                description: "$DESCRIPTION",
                path_parameters: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                request: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
                response: [
                    "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                    ...
                ],
            },
        }
    }
}
 */