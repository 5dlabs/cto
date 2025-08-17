# Task 16: Controller Template Loading - Acceptance Criteria

## Functional Requirements

### ✅ Agent Template Mapping
- [ ] **Rex/Blaze Mapping**: `5DLabs-Rex` and `5DLabs-Blaze` both map to `container-rex.sh.hbs`
- [ ] **Cleo Mapping**: `5DLabs-Cleo` maps to `container-cleo.sh.hbs`  
- [ ] **Tess Mapping**: `5DLabs-Tess` maps to `container-tess.sh.hbs`
- [ ] **Case Sensitivity**: Agent names are matched case-sensitively
- [ ] **Bot Suffix Handling**: GitHub app names with "[bot]" suffix are correctly processed

### ✅ Agent Name Extraction
- [ ] **Direct Format**: Extract "5DLabs-Rex" from "5DLabs-Rex"
- [ ] **Bot Format**: Extract "5DLabs-Rex" from "5DLabs-Rex[bot]"
- [ ] **Error Handling**: Invalid formats return appropriate error messages
- [ ] **Empty Input**: Handle empty or null github_app strings gracefully

### ✅ Template Loading
- [ ] **File Resolution**: Template files are loaded from `templates/` directory
- [ ] **Content Validation**: Loaded templates contain valid shell script headers
- [ ] **Path Security**: Template loading prevents path traversal attacks
- [ ] **File Existence**: Missing template files return clear error messages

### ✅ Fallback Behavior  
- [ ] **Unknown Agent**: Unknown agents default to `container-rex.sh.hbs`
- [ ] **Logging**: Fallback usage is logged with warning level
- [ ] **No Failure**: Fallback never causes system failure
- [ ] **Consistent Behavior**: Same unknown agent always gets same fallback

## Technical Requirements

### ✅ Error Handling
- [ ] **Structured Errors**: Use `anyhow::Result` for error propagation
- [ ] **Error Messages**: Include agent name and template name in error messages
- [ ] **Error Context**: Provide sufficient context for debugging
- [ ] **Non-Panic**: No panics under any input conditions

### ✅ Performance
- [ ] **Template Caching**: Loaded templates are cached to avoid repeated file I/O
- [ ] **Fast Lookup**: Agent to template mapping uses efficient data structures
- [ ] **Memory Efficiency**: String allocations are minimized where possible
- [ ] **Initialization Cost**: Template mapper initialization is performed once

### ✅ Integration
- [ ] **Handlebars Compatible**: Templates work with existing Handlebars compilation
- [ ] **Task Processing**: Integration with existing task execution pipeline
- [ ] **Backward Compatible**: Existing workflows continue to function
- [ ] **Configuration**: Template mappings can be configured externally

## Test Coverage

### ✅ Unit Tests
- [ ] **Agent Mapping Tests**: All agent-to-template mappings are tested
- [ ] **Name Extraction Tests**: Various input formats are validated
- [ ] **Error Scenarios**: Invalid inputs and missing files are tested
- [ ] **Fallback Tests**: Unknown agent fallback behavior is verified

### ✅ Integration Tests
- [ ] **End-to-End**: Complete workflow from github_app to executed template
- [ ] **Template Validation**: All template files load and compile successfully
- [ ] **Performance Tests**: Template loading performance meets requirements
- [ ] **Concurrency Tests**: Thread-safe operation under concurrent access

### ✅ Edge Cases
- [ ] **Malformed Input**: Special characters and unusual formats
- [ ] **File System Issues**: Permissions errors and disk space issues
- [ ] **Template Corruption**: Handling of corrupted or invalid template files
- [ ] **Race Conditions**: Safe operation under concurrent modification

## Validation Procedures

### ✅ Manual Testing
1. **Agent Selection Verification**
   ```bash
   # Test with each agent type
   curl -X POST /api/tasks -d '{"github_app": "5DLabs-Rex"}'
   curl -X POST /api/tasks -d '{"github_app": "5DLabs-Cleo[bot]"}'
   ```

2. **Template Content Verification**
   ```bash
   # Verify template files exist and are valid
   ls -la templates/container-*.sh.hbs
   bash -n templates/container-rex.sh.hbs
   ```

3. **Error Handling Verification**
   ```bash
   # Test unknown agent handling
   curl -X POST /api/tasks -d '{"github_app": "Unknown-Agent"}'
   ```

### ✅ Automated Testing
1. **Unit Test Execution**
   ```bash
   cargo test --package controller --lib tasks::code::templates
   ```

2. **Integration Test Execution**
   ```bash
   cargo test --test template_integration
   ```

3. **Performance Benchmarks**
   ```bash
   cargo bench --bench template_loading
   ```

## Success Metrics

### ✅ Performance Benchmarks
- [ ] **Template Selection**: < 1ms average response time
- [ ] **Template Loading**: < 10ms for cold load, < 0.1ms for cached
- [ ] **Memory Usage**: < 1MB additional memory footprint
- [ ] **Throughput**: Handle 1000+ template selections per second

### ✅ Reliability Metrics
- [ ] **Error Rate**: < 0.1% error rate under normal conditions
- [ ] **Fallback Rate**: Unknown agent fallback usage is tracked and alerted
- [ ] **Template Availability**: 99.9% template loading success rate
- [ ] **Recovery Time**: < 1 second recovery from temporary failures

### ✅ Code Quality
- [ ] **Test Coverage**: > 95% line coverage for new code
- [ ] **Documentation**: All public functions have comprehensive documentation
- [ ] **Linting**: Code passes all clippy lints without warnings
- [ ] **Formatting**: Code follows rustfmt standards

## Deployment Validation

### ✅ Pre-deployment Checks
- [ ] **Template Files**: All required template files are present in deployment
- [ ] **Configuration**: Agent mappings are correctly configured
- [ ] **Backwards Compatibility**: Existing functionality remains unaffected
- [ ] **Monitoring**: Logging and metrics are properly configured

### ✅ Post-deployment Verification
- [ ] **Agent Distribution**: Verify different agents use different templates
- [ ] **Error Monitoring**: No unexpected errors in logs
- [ ] **Performance Impact**: No degradation in overall system performance
- [ ] **Template Usage**: Confirm all templates are being used appropriately

## Rollback Criteria

### ✅ Automatic Rollback Triggers
- [ ] **Error Rate Spike**: > 5% error rate in template selection
- [ ] **Performance Degradation**: > 50% increase in response time
- [ ] **Missing Templates**: Any required template file becomes unavailable
- [ ] **Integration Failure**: Downstream systems report template-related failures

### ✅ Manual Rollback Indicators
- [ ] **Unknown Agent Surge**: Significant increase in fallback usage
- [ ] **Template Compilation Issues**: Handlebars compilation failures
- [ ] **Workflow Disruption**: Any workflow type stops functioning correctly
- [ ] **Resource Exhaustion**: Template caching causes memory issues