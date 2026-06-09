export interface RustDocItem {
  id: string;
  name: string;
  kind: 'struct' | 'function' | 'trait' | 'module' | 'enum';
  docs: string | null;
  attrs: string[];
  visibility: 'public' | 'crate' | 'private';
  span: {
    filename: string;
    begin: [number, number];
    end: [number, number];
  };
}

export interface MDXFrontMatter {
  title: string;
  description: string;
  sidebar: {
    label: string;
    order: number;
  };
}

export interface MDXOutput {
  frontmatter: MDXFrontMatter;
  content: string;
  filename: string;
}

export interface TransformConfig {
  crateNames: string[];
  outputDir: string;
  baseUrl: string;
  moduleMapping: Record<string, { order: number; description: string }>;
}