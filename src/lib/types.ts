// TypeScript types matching Rust structs

export interface BibliographyInfo {
  path: string;
  created_at: string;
  updated_at: string;
  description?: string;
}

export interface Setting {
  bibliographies: Record<string, BibliographyInfo>;
}

// Chunk type matching biblatex's Chunk enum with serde
export type Chunk =
  | { Normal: string }
  | { Verbatim: string }
  | { Math: string };

// Helper to get chunk value
export function getChunkValue(chunk: Chunk): string {
  if ('Normal' in chunk) return chunk.Normal;
  if ('Verbatim' in chunk) return chunk.Verbatim;
  if ('Math' in chunk) return chunk.Math;
  return '';
}

export interface Reference {
  cite_key: string;
  source: string;
  type_: EntryType;
  author?: string[];
  title?: Chunk[];
  journal?: string;
  year?: number;
  full_journal?: string;
  volume?: number;
  number?: string;
  pages?: { start: number; end: number };
  note?: Chunk[];
  doi?: string;
  mrclass?: string;
  publisher?: string[];
  isbn?: string;
  series?: string;
  url?: string;
  file?: string;
  abstract_?: Chunk[];
  edition?: number;
  issue?: Chunk[];
  book_pages?: string;
  school?: string;
  address?: string;
  book_title?: Chunk[];
  editor?: [string, string][];
  month?: string;
  organization?: string[];
  institution?: string;
  eprint?: string;
  archive_prefix?: string;
  arxiv_primary_class?: string;
  how_published?: string;
}

export type EntryType =
  | 'Article'
  | 'Book'
  | 'Booklet'
  | 'InBook'
  | 'InCollection'
  | 'InProceedings'
  | 'Manual'
  | 'MastersThesis'
  | 'Misc'
  | 'PhdThesis'
  | 'Proceedings'
  | 'TechReport'
  | 'Thesis'
  | 'Unpublished'
  | { Unknown: string };

// Filter types
export type FilterField = 'Author' | 'Title' | 'Journal' | 'Year' | 'All';
export type FilterType =
  | 'Book'
  | 'Article'
  | 'Thesis'
  | 'TechReport'
  | 'Misc'
  | 'Booklet'
  | 'InBook'
  | 'InCollection'
  | 'InProceedings'
  | 'All';
