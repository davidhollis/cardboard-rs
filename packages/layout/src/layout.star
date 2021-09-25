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

def solid(color):
    return struct(
        type = "fill",
        filltype = "solid",
        color = color,
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
