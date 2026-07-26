-- Admin-authored email templates.
--
-- Marketing/notification emails are designed in the sitwego-admin app using the
-- Unlayer editor (`react-email-editor`). On save the admin plane stores two
-- artefacts here:
--   * `design_json` — Unlayer's design document, so a template can be re-opened
--     and edited later; never used to render.
--   * `html_src`    — the HTML exported by `editor.exportHtml()`, a complete
--     email whose `{{ merge_tag }}`s are jinja variables. This is what the
--     backend renders (via minijinja) at send time.
--
-- Code-owned *system* emails (OTP, password reset) do NOT live here — they are
-- compile-time askama templates in the `email_api` crate.
CREATE TABLE email_templates (
    -- Internal surrogate key (ULID), consistent with the rest of the schema.
    id            VARCHAR(26) NOT NULL PRIMARY KEY,

    -- Stable identifier referenced by calling code, e.g. 'ride_receipt'.
    slug          VARCHAR     NOT NULL,

    -- Human-facing label shown in the admin template list.
    name          VARCHAR     NOT NULL,

    -- minijinja sources. `subject_src` is the subject line; `html_src` is the
    -- Unlayer-exported HTML document. Both may contain `{{ variable }}` tags.
    subject_src   TEXT        NOT NULL,
    html_src      TEXT        NOT NULL,

    -- Unlayer design document, retained so the editor can re-hydrate the design.
    design_json   JSONB       NOT NULL DEFAULT '{}'::jsonb,

    -- Declared variable contract: the merge tags this template is allowed to use,
    -- e.g. [{"name":"rider_name","required":true,"example":"Alex"}]. Feeds the
    -- editor's `mergeTags` option and lets the backend validate before sending.
    variables     JSONB       NOT NULL DEFAULT '[]'::jsonb,

    -- Only published templates may be sent; drafts are editor-only.
    is_published  BOOLEAN     NOT NULL DEFAULT FALSE,

    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- One row per slug: calling code resolves a template by its stable slug.
    CONSTRAINT uq_email_templates_slug UNIQUE (slug)
);
