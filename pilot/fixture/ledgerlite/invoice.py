"""Invoice totals and rendering; see SPEC.md sections 2 and 3."""


def invoice_total(lines, tax_rate_bp, discount_code=None):
    raise NotImplementedError


def render_tags(tags):
    return ", ".join(tags)
