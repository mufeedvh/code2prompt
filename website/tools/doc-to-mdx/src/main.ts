import * as fs from 'fs-extra';
import * as path from 'path';
import { RustDocParser } from './parser';
import { MDXGenerator } from './generator';
import { TransformConfig } from './types';

async function main() {
  try {
    console.log('🚀 Starting rustdoc to MDX transformation...');
    
    // ~~~ Load configuration ~~~
    const config: TransformConfig = await fs.readJson('./config.json');
    
    // ~~~ Generate rustdoc JSON ~~~
    console.log('📚 Generating rustdoc JSON...');
    await generateRustdocJSON();
    
    // ~~~ Process each crate ~~~
    for (const crateName of config.crateNames) {
      console.log(`🔄 Processing crate: ${crateName}`);
      
      const jsonPath = `../../../target/doc/${crateName.replace('-', '_')}.json`;
      const parser = new RustDocParser(jsonPath);
      
      await parser.load();
      const items = parser.extractPublicItems();
      const moduleGroups = parser.groupByModule(items);
      
      // ~~~ Generate MDX files ~~~
      const generator = new MDXGenerator(config);
      
      await fs.ensureDir(config.outputDir);
      
      for (const [moduleName, moduleItems] of moduleGroups) {
        const mdx = generator.generateModuleMDX(moduleName, moduleItems);
        const outputPath = path.join(config.outputDir, mdx.filename);
        
        const fileContent = [
          '---',
          `title: "${mdx.frontmatter.title}"`,
          `description: "${mdx.frontmatter.description}"`,
          'sidebar:',
          `  label: "${mdx.frontmatter.sidebar.label}"`,
          `  order: ${mdx.frontmatter.sidebar.order}`,
          '---',
          '',
          mdx.content
        ].join('\n');
        
        await fs.writeFile(outputPath, fileContent);
        console.log(`✅ Generated: ${mdx.filename}`);
      }
    }
    
    console.log('🎉 Transformation complete!');
    
  } catch (error) {
    console.error('❌ Error:', error);
    process.exit(1);
  }
}

async function generateRustdocJSON() {
  const { spawn } = require('child_process');
  
  return new Promise<void>((resolve, reject) => {
    const cargo = spawn('cargo', [
      '+nightly',
      'rustdoc',
      '--lib',
      '-p', 'code2prompt_core',
      '--',
      '-Z', 'unstable-options',
      '--output-format', 'json'
    ], {
      cwd: '../../../',
      stdio: ['inherit', 'pipe', 'pipe']
    });
    
    cargo.on('close', (code: number) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`cargo rustdoc failed with code ${code}`));
      }
    });
  });
}

if (require.main === module) {
  main().catch(console.error);
}