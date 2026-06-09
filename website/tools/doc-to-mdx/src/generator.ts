import { RustDocItem, MDXOutput, MDXFrontMatter, TransformConfig } from './types';

export class MDXGenerator {
  constructor(private config: TransformConfig) {}

  generateModuleMDX(moduleName: string, items: RustDocItem[]): MDXOutput {
    const frontmatter = this.generateFrontmatter(moduleName);
    const content = this.generateContent(moduleName, items);
    
    return {
      frontmatter,
      content,
      filename: `${moduleName}.mdx`
    };
  }

  private generateFrontmatter(moduleName: string): MDXFrontMatter {
    const moduleConfig = this.config.moduleMapping[moduleName];
    
    return {
      title: `${this.capitalizeFirst(moduleName)} API`,
      description: moduleConfig?.description || `${moduleName} module API reference`,
      sidebar: {
        label: this.capitalizeFirst(moduleName),
        order: moduleConfig?.order || 99
      }
    };
  }

  private generateContent(moduleName: string, items: RustDocItem[]): string {
    let content = `# ${this.capitalizeFirst(moduleName)} API\n\n`;
    
    // ~~~ Group items by type ~~~
    const structs = items.filter(item => item.kind === 'struct');
    const functions = items.filter(item => item.kind === 'function');
    const traits = items.filter(item => item.kind === 'trait');
    
    // ~~~ Generate sections ~~~
    if (structs.length > 0) {
      content += this.generateStructsSection(structs);
    }
    
    if (functions.length > 0) {
      content += this.generateFunctionsSection(functions);
    }
    
    if (traits.length > 0) {
      content += this.generateTraitsSection(traits);
    }
    
    return content;
  }

  private generateStructsSection(structs: RustDocItem[]): string {
    let section = '\n## Structures\n\n';
    
    for (const struct of structs) {
      section += `### \`${struct.name}\`\n\n`;
      
      if (struct.docs) {
        section += `${this.formatDocstring(struct.docs)}\n\n`;
      }
    }
    
    return section;
  }

  private generateFunctionsSection(functions: RustDocItem[]): string {
    let section = '\n## Functions\n\n';
    
    for (const func of functions) {
      section += `### \`${func.name}()\`\n\n`;
      
      if (func.docs) {
        section += `${this.formatDocstring(func.docs)}\n\n`;
      }
    }
    
    return section;
  }

  private generateTraitsSection(traits: RustDocItem[]): string {
    let section = '\n## Traits\n\n';
    
    for (const trait of traits) {
      section += `### \`${trait.name}\`\n\n`;
      
      if (trait.docs) {
        section += `${this.formatDocstring(trait.docs)}\n\n`;
      }
    }
    
    return section;
  }

  private formatDocstring(docs: string): string {
    // Convert rustdoc formatting to MDX-compatible format
    return docs
      .replace(/^\/\/\/ ?/gm, '') // Remove doc comment markers
      .replace(/^\/\*\*/, '').replace(/\*\/$/, '') // Remove block comment markers
      .trim();
  }

  private capitalizeFirst(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }
}