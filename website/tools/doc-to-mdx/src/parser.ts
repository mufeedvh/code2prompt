import * as fs from 'fs-extra';
import * as path from 'path';
import { RustDocItem } from './types';

export class RustDocParser {
  private docData: any;

  constructor(private jsonPath: string) {}

  async load(): Promise<void> {
    if (!fs.existsSync(this.jsonPath)) {
      throw new Error(`Rustdoc JSON file not found: ${this.jsonPath}`);
    }
    
    this.docData = await fs.readJson(this.jsonPath);
  }

  extractPublicItems(): RustDocItem[] {
    const items: RustDocItem[] = [];
    
    // ~~~ Process crate items ~~~
    for (const [id, item] of Object.entries(this.docData.index)) {
      const typedItem = item as any;
      
      if (typedItem.visibility !== 'public') {
        continue;
      }
      
      items.push({
        id,
        name: typedItem.name || 'unnamed',
        kind: typedItem.kind,
        docs: typedItem.docs,
        attrs: typedItem.attrs || [],
        visibility: typedItem.visibility,
        span: typedItem.span
      });
    }
    
    return items;
  }

  groupByModule(items: RustDocItem[]): Map<string, RustDocItem[]> {
    const modules = new Map<string, RustDocItem[]>();
    
    for (const item of items) {
      const moduleName = this.extractModuleName(item);
      
      if (!modules.has(moduleName)) {
        modules.set(moduleName, []);
      }
      
      modules.get(moduleName)!.push(item);
    }
    
    return modules;
  }

  private extractModuleName(item: RustDocItem): string {
    // Extract module name from span filename
    const filename = path.basename(item.span.filename, '.rs');
    return filename === 'lib' ? 'core' : filename;
  }
}