# Import the Rust module
from . import code2prompt_rs as rust_sdk
from pathlib import Path

class RenderedPrompt:
    def __init__(self, prompt, token_count, directory, model_info):
        self.prompt = prompt
        self.token_count = token_count
        self.directory = directory
        self.model_info = model_info

class Code2Prompt:
    def __init__(self, path, include_patterns=None, exclude_patterns=None, 
                 include_priority=False, line_numbers=False, absolute_paths=False,
                 full_directory_tree=False, code_blocks=True, follow_symlinks=False, include_hidden=False):
        """
        Initialize a Code2Prompt configuration for generating prompts from code.
        
        Args:
            path: Path to the code directory
            include_patterns: List of glob patterns for files to include
            exclude_patterns: List of glob patterns for files to exclude
            include_priority: Whether to prioritize include patterns over exclude
            line_numbers: Whether to include line numbers in the output
            absolute_paths: Whether to use absolute paths in the output
            full_directory_tree: Whether to include the full directory tree
            code_blocks: Whether to wrap code in markdown code blocks
            follow_symlinks: Whether to follow symlinks
            include_hidden: Whether to include hidden files (default is False)
        """
        # Stocker la configuration
        self.path = Path(path)
        self.include_patterns = include_patterns or []
        self.exclude_patterns = exclude_patterns or []
        self.include_priority = include_priority
        self.line_numbers = line_numbers
        self.absolute_paths = absolute_paths
        self.full_directory_tree = full_directory_tree
        self.code_blocks = code_blocks
        self.follow_symlinks = follow_symlinks
        self.include_hidden = include_hidden
        
        # Initializer une session uniquement quand nécessaire
        self._session = None

    def session(self) -> rust_sdk.PyCode2PromptSession:
        """
        Create a PyCode2PromptSession with the current configuration.
        """
        # Créer la session Rust avec la configuration actuelle
        session = rust_sdk.PyCode2PromptSession(str(self.path))
        
        # Appliquer toutes les configurations
        if self.include_patterns:
            session = session.include(self.include_patterns)
        if self.exclude_patterns:
            session = session.exclude(self.exclude_patterns)
            
        session = session.include_priority(self.include_priority)
        session = session.with_line_numbers(self.line_numbers)
        session = session.with_absolute_paths(self.absolute_paths)
        session = session.with_full_directory_tree(self.full_directory_tree)
        session = session.with_code_blocks(self.code_blocks)
        session = session.follow_symlinks(self.follow_symlinks)
        session = session.include_hidden(self.include_hidden)
        
        return session
    
    def generate(self, template=None, encoding=None) -> RenderedPrompt:
        """
        Generate a prompt from the code.
        
        Args:
            template: Optional template string to use
            encoding: Token encoding to use (e.g., 'cl100k', 'gpt2')
        
        Returns:
            String containing the generated prompt
        """
        # Apply optional configurations
        session = self._session or self.session()
        
        if encoding:
            session = session.with_token_encoding(encoding)
        
        if template:
            session = session.with_template(template)
            
        # Generate the prompt
        result = session.generate()
        
        # Get token count
        try:
            token_count = session.token_count()
        except Exception:
            token_count = 0
            
        # Return a dictionary with results
        return RenderedPrompt(
            prompt=result,
            token_count=token_count,
            directory=self.path,
            model_info=session.info()
        )
    
    def token_count(self, encoding=None):
        """Get token count for the prompt with specified encoding."""
        session = self._session or self.session()
        if encoding:
            session = session.with_token_encoding(encoding)
        return session.token_count()
    
    def info(self):
        """Get information about the current session."""
        session = self._session or self.session()
        return session.info()