// Copyright 2026 Oxide Computer Company

//! Twin-spec equivalence: the same API authored as `OpenAPI` 3.0 and as
//! `OpenAPI` 3.1 must generate byte-identical clients. This is the
//! acceptance bar for the 3.1 frontend — typify embeds schema values in
//! doc comments, so anything short of byte-equal lowering shows up here.

use indoc::indoc;
use progenitor_impl::Generator;

/// The 3.0 spelling: `nullable`, single-value `enum`, boolean
/// `exclusiveMinimum`, `format: binary` upload, `Label` as a component.
const TWIN_V30: &str = indoc! {r##"
    {
      "openapi": "3.0.3",
      "info": {
        "title": "Twin",
        "version": "1.0.0",
        "description": "Twin-spec equivalence fixture"
      },
      "paths": {
        "/things/{id}": {
          "get": {
            "operationId": "get_thing",
            "parameters": [
              {
                "name": "id",
                "in": "path",
                "required": true,
                "schema": { "type": "string", "format": "uuid" }
              },
              {
                "name": "verbose",
                "in": "query",
                "schema": { "type": "boolean" }
              }
            ],
            "responses": {
              "200": {
                "description": "ok",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Thing" }
                  }
                }
              },
              "default": {
                "description": "error",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Error" }
                  }
                }
              }
            }
          }
        },
        "/things": {
          "post": {
            "operationId": "create_thing",
            "requestBody": {
              "required": true,
              "content": {
                "application/json": {
                  "schema": { "$ref": "#/components/schemas/CreateThing" }
                }
              }
            },
            "responses": {
              "201": {
                "description": "created",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Thing" }
                  }
                }
              }
            }
          }
        },
        "/pets": {
          "get": {
            "operationId": "get_pet",
            "responses": {
              "200": {
                "description": "ok",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Pet" }
                  }
                }
              }
            }
          }
        },
        "/upload": {
          "post": {
            "operationId": "upload_blob",
            "requestBody": {
              "content": {
                "application/octet-stream": {
                  "schema": { "type": "string", "format": "binary" }
                }
              }
            },
            "responses": { "204": { "description": "uploaded" } }
          }
        },
        "/note": {
          "put": {
            "operationId": "set_note",
            "requestBody": {
              "content": {
                "text/plain": { "schema": { "type": "string" } }
              }
            },
            "responses": { "204": { "description": "saved" } }
          }
        }
      },
      "components": {
        "schemas": {
          "Thing": {
            "type": "object",
            "required": ["id", "kind"],
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "kind": { "type": "string", "enum": ["fixed"] },
              "note": { "type": "string", "nullable": true },
              "count": { "type": "integer", "minimum": 0 },
              "weight": {
                "type": "number",
                "minimum": 0.0,
                "exclusiveMinimum": true
              },
              "labels": {
                "type": "array",
                "items": { "$ref": "#/components/schemas/Label" }
              },
              "either": {
                "nullable": true,
                "oneOf": [
                  { "$ref": "#/components/schemas/Label" },
                  { "$ref": "#/components/schemas/Error" }
                ]
              }
            }
          },
          "Label": {
            "type": "object",
            "required": ["text"],
            "properties": { "text": { "type": "string" } }
          },
          "CreateThing": {
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "type": "string" } }
          },
          "Error": {
            "type": "object",
            "required": ["message"],
            "properties": { "message": { "type": "string" } }
          },
          "Pet": {
            "type": "object",
            "required": ["petType"],
            "properties": { "petType": { "type": "string" } },
            "discriminator": { "propertyName": "petType" }
          },
          "Dog": {
            "allOf": [
              { "$ref": "#/components/schemas/Pet" },
              {
                "type": "object",
                "properties": { "bark": { "type": "boolean" } }
              }
            ]
          }
        }
      }
    }
"##};

/// The same API in 3.1 spelling: type arrays with `null`, `const`,
/// numeric `exclusiveMinimum`, a schema-less binary upload, `Label` as a
/// `$defs` member, and a `oneOf` with an explicit null branch.
const TWIN_V31: &str = indoc! {r##"
    {
      "openapi": "3.1.0",
      "info": {
        "title": "Twin",
        "version": "1.0.0",
        "description": "Twin-spec equivalence fixture"
      },
      "paths": {
        "/things/{id}": {
          "get": {
            "operationId": "get_thing",
            "parameters": [
              {
                "name": "id",
                "in": "path",
                "required": true,
                "schema": { "type": "string", "format": "uuid" }
              },
              {
                "name": "verbose",
                "in": "query",
                "schema": { "type": "boolean" }
              }
            ],
            "responses": {
              "200": {
                "description": "ok",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Thing" }
                  }
                }
              },
              "default": {
                "description": "error",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Error" }
                  }
                }
              }
            }
          }
        },
        "/things": {
          "post": {
            "operationId": "create_thing",
            "requestBody": {
              "required": true,
              "content": {
                "application/json": {
                  "schema": { "$ref": "#/components/schemas/CreateThing" }
                }
              }
            },
            "responses": {
              "201": {
                "description": "created",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Thing" }
                  }
                }
              }
            }
          }
        },
        "/pets": {
          "get": {
            "operationId": "get_pet",
            "responses": {
              "200": {
                "description": "ok",
                "content": {
                  "application/json": {
                    "schema": { "$ref": "#/components/schemas/Pet" }
                  }
                }
              }
            }
          }
        },
        "/upload": {
          "post": {
            "operationId": "upload_blob",
            "requestBody": {
              "content": {
                "application/octet-stream": {}
              }
            },
            "responses": { "204": { "description": "uploaded" } }
          }
        },
        "/note": {
          "put": {
            "operationId": "set_note",
            "requestBody": {
              "content": {
                "text/plain": { "schema": { "type": "string" } }
              }
            },
            "responses": { "204": { "description": "saved" } }
          }
        }
      },
      "components": {
        "schemas": {
          "Thing": {
            "type": "object",
            "required": ["id", "kind"],
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "kind": { "type": "string", "const": "fixed" },
              "note": { "type": ["string", "null"] },
              "count": { "type": "integer", "minimum": 0 },
              "weight": { "type": "number", "exclusiveMinimum": 0.0 },
              "labels": {
                "type": "array",
                "items": { "$ref": "#/$defs/Label" }
              },
              "either": {
                "oneOf": [
                  { "$ref": "#/components/schemas/Label" },
                  { "$ref": "#/components/schemas/Error" },
                  { "type": "null" }
                ]
              }
            },
            "$defs": {
              "Label": {
                "type": "object",
                "required": ["text"],
                "properties": { "text": { "type": "string" } }
              }
            }
          },
          "CreateThing": {
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "type": "string" } }
          },
          "Error": {
            "type": "object",
            "required": ["message"],
            "properties": { "message": { "type": "string" } }
          },
          "Pet": {
            "type": "object",
            "required": ["petType"],
            "properties": { "petType": { "type": "string" } },
            "discriminator": { "propertyName": "petType" }
          },
          "Dog": {
            "allOf": [
              { "$ref": "#/components/schemas/Pet" },
              {
                "type": "object",
                "properties": { "bark": { "type": "boolean" } }
              }
            ]
          }
        }
      }
    }
"##};

fn generate(spec_text: &str) -> String {
    let spec = progenitor_impl::parse_openapi_str(spec_text).unwrap();
    let mut generator = Generator::default();
    let content = generator.generate_tokens(&spec).unwrap();

    let rustfmt_config = rustfmt_wrapper::config::Config {
        format_strings: Some(true),
        normalize_doc_attributes: Some(true),
        wrap_comments: Some(true),
        ..Default::default()
    };
    progenitor_impl::space_out_items(
        rustfmt_wrapper::rustfmt_config(rustfmt_config, content).unwrap(),
    )
    .unwrap()
}

#[test]
fn twin_specs_generate_identical_clients() {
    let output_v31 = generate(TWIN_V31);
    expectorate::assert_contents("tests/output/src/twin_v31_positional.rs", &output_v31);

    let output_v30 = generate(TWIN_V30);
    assert_eq!(
        output_v30, output_v31,
        "3.0 and 3.1 twins must generate identical code"
    );
}
