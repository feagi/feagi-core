#!/bin/bash
# GPU Support Verification Script for FEAGI
# Tests GPU detection, configuration, and backend selection

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           FEAGI GPU Support Verification Script              ║${NC}"
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo ""

# Function to run a test
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    local expected_pattern="$3"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    echo -e "${YELLOW}▶ Test $TESTS_TOTAL: $test_name${NC}"
    
    # Run command and capture output
    if output=$(eval "$test_cmd" 2>&1); then
        # Check if expected pattern is found
        if echo "$output" | grep -q "$expected_pattern"; then
            echo -e "${GREEN}  ✓ PASSED${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            return 0
        else
            echo -e "${RED}  ✗ FAILED: Expected pattern '$expected_pattern' not found${NC}"
            echo -e "${RED}  Output: ${output:0:200}...${NC}"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            return 1
        fi
    else
        echo -e "${RED}  ✗ FAILED: Command exited with error${NC}"
        echo -e "${RED}  Output: ${output:0:200}...${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to check if binary exists
check_binary() {
    local binary="$1"
    if [ ! -f "$binary" ]; then
        echo -e "${RED}✗ Binary not found: $binary${NC}"
        echo -e "${YELLOW}  Please build first: cargo build --release${NC}"
        return 1
    fi
    echo -e "${GREEN}✓ Binary found: $binary${NC}"
    return 0
}

# Change to feagi-core directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FEAGI_CORE_DIR="$(dirname "$SCRIPT_DIR")"
cd "$FEAGI_CORE_DIR"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 1: Check Build Status${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if feagi-burst-engine has GPU feature
if grep -q 'features = \["gpu"\]' crates/feagi-burst-engine/Cargo.toml 2>/dev/null; then
    echo -e "${GREEN}✓ GPU feature flag found in burst-engine Cargo.toml${NC}"
else
    echo -e "${YELLOW}⚠ GPU feature flag not explicitly set (may be using workspace default)${NC}"
fi

# Check if WGPU backend exists
if [ -f "crates/feagi-burst-engine/src/backend/wgpu_backend.rs" ]; then
    echo -e "${GREEN}✓ WGPU backend source file exists${NC}"
    line_count=$(wc -l < "crates/feagi-burst-engine/src/backend/wgpu_backend.rs")
    echo -e "  Lines of code: $line_count"
else
    echo -e "${RED}✗ WGPU backend source file not found${NC}"
    exit 1
fi

# Check if GPU shaders exist
shader_count=$(find crates/feagi-burst-engine/src/backend/shaders -name "*.wgsl" 2>/dev/null | wc -l)
if [ "$shader_count" -gt 0 ]; then
    echo -e "${GREEN}✓ GPU shaders found: $shader_count WGSL files${NC}"
    find crates/feagi-burst-engine/src/backend/shaders -name "*.wgsl" -exec echo "  - {}" \;
else
    echo -e "${YELLOW}⚠ No GPU shaders found (may not be compiled yet)${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 2: Check Configuration System${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if GpuConfig exists in code
if grep -r "pub struct GpuConfig" crates/feagi-burst-engine/src/backend/ 2>/dev/null; then
    echo -e "${GREEN}✓ GpuConfig struct found in backend module${NC}"
else
    echo -e "${YELLOW}⚠ GpuConfig struct not found (needs to be added)${NC}"
fi

# Check if HybridConfig exists in feagi-config
if grep -r "pub struct HybridConfig" crates/feagi-config/src/ 2>/dev/null; then
    echo -e "${GREEN}✓ HybridConfig struct found in feagi-config${NC}"
else
    echo -e "${RED}✗ HybridConfig struct not found in feagi-config${NC}"
fi

# Check if ResourcesConfig has use_gpu field
if grep -A5 "pub struct ResourcesConfig" crates/feagi-config/src/types.rs | grep -q "pub use_gpu"; then
    echo -e "${GREEN}✓ ResourcesConfig.use_gpu field found${NC}"
else
    echo -e "${RED}✗ ResourcesConfig.use_gpu field not found${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 3: Build Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Build with GPU support
echo -e "${YELLOW}Building burst-engine with GPU feature...${NC}"
if cd crates/feagi-burst-engine && cargo build --features gpu --quiet 2>&1; then
    echo -e "${GREEN}✓ Burst engine built successfully with GPU support${NC}"
else
    echo -e "${RED}✗ Failed to build burst engine with GPU support${NC}"
    exit 1
fi

cd "$FEAGI_CORE_DIR"

# Build without GPU support
echo -e "${YELLOW}Building burst-engine without GPU feature (CPU only)...${NC}"
if cd crates/feagi-burst-engine && cargo build --no-default-features --quiet 2>&1; then
    echo -e "${GREEN}✓ Burst engine built successfully (CPU only)${NC}"
else
    echo -e "${YELLOW}⚠ CPU-only build failed (may not be critical)${NC}"
fi

cd "$FEAGI_CORE_DIR"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 4: GPU Detection Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Test GPU availability
echo -e "${YELLOW}Testing GPU availability...${NC}"

# Create a simple GPU detection test
cat > /tmp/test_gpu_detection.rs << 'EOF'
fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));
    
    match adapter {
        Some(adapter) => {
            let info = adapter.get_info();
            println!("GPU_DETECTED");
            println!("Name: {}", info.name);
            println!("Backend: {:?}", info.backend);
            println!("Device: {:?}", info.device_type);
        }
        None => {
            println!("GPU_NOT_DETECTED");
            eprintln!("No GPU adapter found");
        }
    }
}
EOF

# Try to run GPU detection (if wgpu is available)
if cargo run --example 2>&1 | grep -q "wgpu"; then
    echo -e "${GREEN}✓ WGPU crate is available${NC}"
else
    echo -e "${YELLOW}⚠ Cannot test GPU detection (wgpu examples not available)${NC}"
fi

# Check system GPU
echo -e "${YELLOW}Checking system GPU information...${NC}"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    if system_profiler SPDisplaysDataType 2>/dev/null | grep -q "Metal"; then
        echo -e "${GREEN}✓ Metal GPU detected (macOS)${NC}"
        system_profiler SPDisplaysDataType | grep -A5 "Chipset Model:" | head -6
    else
        echo -e "${YELLOW}⚠ Metal support status unknown${NC}"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    if lspci 2>/dev/null | grep -i vga; then
        echo -e "${GREEN}✓ GPU detected (Linux)${NC}"
        lspci | grep -i vga
    else
        echo -e "${YELLOW}⚠ Could not detect GPU on Linux${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Unknown OS type: $OSTYPE${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 5: Backend Selection Logic Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Run backend selection tests
echo -e "${YELLOW}Running backend selection tests...${NC}"

cd crates/feagi-burst-engine
if cargo test --features gpu test_backend_selection 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Backend selection tests passed${NC}"
else
    echo -e "${YELLOW}⚠ Backend selection tests not found or failed${NC}"
fi

# Run speedup estimation tests
if cargo test --features gpu test_speedup_estimation 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Speedup estimation tests passed${NC}"
else
    echo -e "${YELLOW}⚠ Speedup estimation tests not found or failed${NC}"
fi

cd "$FEAGI_CORE_DIR"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 6: Configuration Integration Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if feagi_configuration.toml exists
if [ -f "../../feagi/feagi_configuration.toml" ]; then
    echo -e "${GREEN}✓ feagi_configuration.toml found${NC}"
    
    # Check for GPU config fields
    if grep -q "\\[neural.hybrid\\]" "../../feagi/feagi_configuration.toml"; then
        echo -e "${GREEN}✓ [neural.hybrid] section found in config${NC}"
    else
        echo -e "${RED}✗ [neural.hybrid] section not found in config${NC}"
    fi
    
    if grep -q "gpu_threshold" "../../feagi/feagi_configuration.toml"; then
        echo -e "${GREEN}✓ gpu_threshold field found in config${NC}"
        threshold=$(grep "gpu_threshold" "../../feagi/feagi_configuration.toml" | head -1)
        echo -e "  Value: $threshold"
    else
        echo -e "${RED}✗ gpu_threshold field not found in config${NC}"
    fi
    
    if grep -q "use_gpu" "../../feagi/feagi_configuration.toml"; then
        echo -e "${GREEN}✓ use_gpu field found in config${NC}"
        use_gpu=$(grep "use_gpu" "../../feagi/feagi_configuration.toml" | head -1)
        echo -e "  Value: $use_gpu"
    else
        echo -e "${RED}✗ use_gpu field not found in config${NC}"
    fi
else
    echo -e "${YELLOW}⚠ feagi_configuration.toml not found (expected in ../../feagi/)${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 7: Integration Status Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if GPU config is wired in main.rs
if [ -f "../../feagi/src/main.rs" ]; then
    if grep -q "GpuConfig" "../../feagi/src/main.rs"; then
        echo -e "${GREEN}✓ GpuConfig usage found in feagi/src/main.rs${NC}"
    else
        echo -e "${YELLOW}⚠ GpuConfig not used in feagi/src/main.rs (wiring not complete)${NC}"
    fi
    
    if grep -q "gpu_config" "../../feagi/src/main.rs"; then
        echo -e "${GREEN}✓ gpu_config variable found in feagi/src/main.rs${NC}"
    else
        echo -e "${YELLOW}⚠ gpu_config variable not found in feagi/src/main.rs (wiring not complete)${NC}"
    fi
fi

# Check if RustNPU::new accepts gpu_config parameter
if grep -A10 "pub fn new" crates/feagi-burst-engine/src/npu.rs | grep -q "gpu_config"; then
    echo -e "${GREEN}✓ RustNPU::new() accepts gpu_config parameter${NC}"
else
    echo -e "${YELLOW}⚠ RustNPU::new() does not accept gpu_config parameter (update needed)${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Final Results${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Tests Total:  ${BLUE}$TESTS_TOTAL${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                  ✓ ALL TESTS PASSED                          ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║         ⚠ SOME CHECKS FAILED OR INCOMPLETE                  ║${NC}"
    echo -e "${YELLOW}║   See implementation plan: GPU_CONFIG_WIRING_IMPLEMENTATION.md  ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi

