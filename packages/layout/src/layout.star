# ---------------------
# -- reference types --
# ---------------------

def ref(path):
    return struct(
        type = "ref",
        path = path
    )


# -------------------
# -- card geometry --
# -------------------

def insets(top, right, bottom, left):
    return struct(
        type = "insets",
        top = top,
        right = right,
        bottom = bottom,
        left = left,
    )

def uniform(length):
    return insets(
        top = length,
        right = length,
        bottom = length,
        left = length,
    )

def geometry(units, width, height, cut = uniform(0), safe = uniform(0)):
    return struct(
        type = "geometry",
        units = units,
        width = width,
        height = height,
        cut = cut,
        safe = safe,
    )

# ------------
# -- colors --
# ------------

def named(name):
    return struct(
        type = "color",
        color_type = "named",
        name = name,
    )

def rgba(red, green, blue, alpha):
    return struct(
        type = "color",
        color_type = "rgba",
        red = red,
        green = green,
        blue = blue,
        alpha = alpha,
    )

# -------------
# -- strokes --
# -------------

def stroke(style, color, width):
    return struct(
        type = "stroke",
        style = style,
        color = color,
        width = width,
    )

# -----------
# -- fills --
# -----------

def solid(color):
    return struct(
        type = "fill",
        fill_type = "solid",
        color = color,
    )

def no_fill():
    return struct(
        type = "fill",
        fill_type = "none",
    )

# ------------
# -- shapes --
# ------------

def rectangle(x, y, width, height, corner_radius = 0):
    return struct(
        type = "shape",
        shape_type = "rectangle",
        x = x,
        y = y,
        width = width,
        height = height,
        corner_radius = corner_radius,
    )

# --------------
# -- elements --
# --------------

def shape(shape, stroke = stroke("solid", named("black"), 1), fill = no_fill()):
    return struct(
        type = "element",
        element_type = "shape",
        shape = shape,
        stroke = stroke,
        fill = fill,
    )

# ----------------------------
# -- the layout constructor --
# ----------------------------

def layout(geometry, background, elements = []):
    return struct(
        type = "layout",
        geometry = geometry,
        background = background,
        elements = elements,
    )
