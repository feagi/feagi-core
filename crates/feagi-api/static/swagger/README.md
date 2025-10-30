# Custom Swagger UI Assets

This directory contains custom styling and assets for the FEAGI Swagger UI.

## Migration from Python Implementation

To preserve the custom Swagger UI styling from the Python FastAPI implementation:

### 1. Locate Python Swagger Assets

Find the custom Swagger UI assets in the Python codebase:
```bash
# Common locations:
feagi-py/feagi/api/static/swagger/
feagi-py/static/
feagi-py/feagi/api/templates/swagger/
```

### 2. Copy CSS Files

Copy any custom CSS files to this directory:
```bash
cp /path/to/python/static/swagger/*.css ./custom.css
```

### 3. Copy JavaScript Files

Copy any custom JavaScript (if present):
```bash
cp /path/to/python/static/swagger/*.js ./custom.js
```

### 4. Copy Images/Logos

Copy any images or logos:
```bash
cp /path/to/python/static/swagger/*.{png,jpg,svg} ./
```

### 5. Update Rust Integration

Once assets are copied, update `src/transports/http/server.rs` to serve them:

```rust
use tower_http::services::ServeDir;

// Add static file serving
.nest_service("/static", ServeDir::new("static"))
```

Then configure `utoipa_swagger_ui` to use custom CSS:

```rust
SwaggerUi::new("/swagger-ui")
    .url("/openapi.json", ApiDoc::openapi())
    .config(
        utoipa_swagger_ui::Config::default()
            .custom_css_url("/static/swagger/custom.css")
    )
```

## Current Status

**Implemented:**
- ✅ Directory structure
- ✅ Placeholder `custom.css` with FEAGI color palette
- ✅ Swagger UI integration in HTTP server

**TODO:**
- [ ] Copy actual CSS from Python implementation
- [ ] Copy any custom JavaScript
- [ ] Copy logo/images
- [ ] Test visual consistency with Python version
- [ ] Update `custom.css` with complete styling

## Testing Custom Styling

1. Start the HTTP server
2. Navigate to `http://localhost:8080/swagger-ui/`
3. Verify custom styling is applied
4. Compare with Python version at `http://localhost:8000/docs`
5. Adjust CSS as needed to match

## Custom CSS Variables

The current placeholder uses these CSS variables:

```css
--feagi-primary: #3b82f6      /* Primary brand color */
--feagi-secondary: #8b5cf6    /* Secondary accent */
--feagi-background: #0f172a   /* Page background */
--feagi-surface: #1e293b      /* Card/surface color */
--feagi-text: #f1f5f9         /* Text color */
```

Update these to match the actual FEAGI brand colors from the Python implementation.

