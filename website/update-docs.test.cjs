const { spawn } = require('child_process');
const path = require('path');

// Mock spawn to avoid actual command execution in tests
jest.mock('child_process');

describe('updateAPIDocs', () => {
  let updateAPIDocs;

  beforeAll(async () => {
    // Import CommonJS module
    const module = require('./update-docs.cjs');
    updateAPIDocs = module.updateAPIDocs;
  });

  beforeEach(() => {
    jest.clearAllMocks();
    // Mock successful command execution by default
    spawn.mockImplementation(() => ({
      on: jest.fn((event, callback) => {
        if (event === 'close') {
          // Simulate successful execution (exit code 0)
          setTimeout(() => callback(0), 10);
        }
      })
    }));
  });

  test('executes build and start commands for MDX transformer', async () => {
    await updateAPIDocs();
    
    expect(spawn).toHaveBeenCalledTimes(2);
    
    // First call should be npm run build
    expect(spawn).toHaveBeenNthCalledWith(1, 'npm', ['run', 'build'], {
      stdio: 'inherit',
      cwd: expect.stringContaining('tools/doc-to-mdx')
    });
    
    // Second call should be npm start  
    expect(spawn).toHaveBeenNthCalledWith(2, 'npm', ['start'], {
      stdio: 'inherit', 
      cwd: expect.stringContaining('tools/doc-to-mdx')
    });
  });

  test('throws error when build command fails', async () => {
    // Mock first command (build) to fail with exit code 1
    let callCount = 0;
    spawn.mockImplementation(() => ({
      on: jest.fn((event, callback) => {
        if (event === 'close') {
          callCount++;
          const exitCode = callCount === 1 ? 1 : 0; // First call fails
          setTimeout(() => callback(exitCode), 10);
        }
      })
    }));

    await expect(updateAPIDocs()).rejects.toThrow('npm run build failed with code 1');
  });

  test('throws error when start command fails', async () => {
    // Mock second command (start) to fail with exit code 1  
    let callCount = 0;
    spawn.mockImplementation(() => ({
      on: jest.fn((event, callback) => {
        if (event === 'close') {
          callCount++;
          const exitCode = callCount === 2 ? 1 : 0; // Second call fails
          setTimeout(() => callback(exitCode), 10);
        }
      })
    }));

    await expect(updateAPIDocs()).rejects.toThrow('npm start failed with code 1');
  });
});