# Custom Swagger UI Assets

This directory is reserved for custom assets (images, logos, etc.) for the FEAGI Swagger UI.

## Implementation Status

**✅ COMPLETE**: The Swagger UI theme system has been fully implemented and migrated from the Python (feagi-py) implementation.

## Architecture

The FEAGI Swagger UI uses a **custom HTML template** with embedded CSS and JavaScript, rather than separate static files:

- **Template Location**: `crates/feagi-api/templates/custom-swagger-ui.html`
- **Served At**: `/swagger-ui/`
- **Implementation**: The template is embedded at compile time using Rust's `include_str!` macro

## Features

### Theme System
- **Light Theme**: Clean, modern light color scheme with FEAGI branding
- **Dark Theme**: Default dark theme optimized for reduced eye strain
- **System Theme**: Automatically follows the user's system color scheme preference
- **Theme Persistence**: User's theme choice is saved in browser localStorage
- **Dynamic Switching**: Themes can be switched on-the-fly using the dropdown in the header

### UI Components
- Custom header with FEAGI branding
- Expand/Collapse All buttons for API endpoints
- Quick-load buttons for Essential and Barebones genomes
- Real-time search/filter functionality
- Responsive design

### Color Schemes

**Dark Theme:**
```css
--primary-color: #19b6b5       /* Teal primary */
--background-dark: #121417     /* Deep dark background */
--background-medium: #1f2426   /* Card backgrounds */
--text-primary: #f1f1f1        /* Light text */
```

**Light Theme:**
```css
--primary-color: #007b8a       /* Dark teal */
--background-dark: #ffffff     /* White background */
--background-medium: #f9fafa   /* Light gray cards */
--text-primary: #1e2b33        /* Dark text */
```

## Usage

### For Users
1. Navigate to `http://localhost:8080/swagger-ui/`
2. Use the **Theme** dropdown in the header to switch between:
   - **System**: Follows your OS theme (default)
   - **Dark**: Always use dark theme
   - **Light**: Always use light theme
3. Your choice is automatically saved

### For Developers
The theme system is implemented in the custom HTML template. To modify:

1. **Edit the template**: `crates/feagi-api/templates/custom-swagger-ui.html`
2. **Rebuild the project**: The template is embedded at compile time
3. **Restart the server**: Changes take effect on next server start

### Modifying Colors
Edit the CSS variables in the `<style>` section of the template:
- `:root { ... }` for dark theme (default)
- `[data-theme="light"] { ... }` for light theme

### Adding Custom Assets
If you need to serve static files (images, fonts, etc.), place them in this directory and they will be served at `/static/swagger/`.

## Comparison with Python Implementation

This Rust implementation provides **feature parity** with the Python (feagi-py) Swagger UI:

| Feature | Python (feagi-py) | Rust (feagi-core) |
|---------|-------------------|-------------------|
| Light/Dark Themes | ✅ | ✅ |
| System Theme Detection | ✅ | ✅ |
| Theme Persistence | ✅ | ✅ |
| Custom Branding | ✅ | ✅ |
| Search/Filter | ✅ | ✅ |
| Genome Quick-Load | ✅ | ✅ |
| Responsive Design | ✅ | ✅ |

## Migration Notes

The implementation was migrated from feagi-py with the following improvements:
1. **Simplified deployment**: CSS/JS embedded in template (no separate files)
2. **Better performance**: Template embedded at compile time
3. **Easier maintenance**: Single file contains all custom UI code
4. **Cross-platform**: Works identically on all platforms




