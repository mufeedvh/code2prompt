#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');

async function updateAPIDocs() {
  console.log('🔄 Updating API Reference Documentation...');
  
  // Run MDX transformer
  console.log('📝 Generating API documentation...');
  await runCommand('npm', ['run', 'build'], { cwd: path.join(__dirname, 'tools/doc-to-mdx') });
  await runCommand('npm', ['start'], { cwd: path.join(__dirname, 'tools/doc-to-mdx') });
  
  console.log('✅ API documentation updated successfully!');
  console.log('📍 Generated files: src/content/docs/docs/references/api/');
}

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, { 
      stdio: 'inherit',
      ...options 
    });
    
    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} ${args.join(' ')} failed with code ${code}`));
      }
    });
  });
}

if (require.main === module) {
  updateAPIDocs().catch((error) => {
    console.error('❌ Failed to update API docs:', error.message);
    process.exit(1);
  });
}

module.exports = { updateAPIDocs };