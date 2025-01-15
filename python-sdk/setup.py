from setuptools import setup, find_packages

setup(
    name="code2prompt",
    version="2.0.0",
    packages=find_packages(),
    install_requires=[],
    author="Mufeed VH",
    author_email="contact@mufeedvh.com",
    description="Python bindings for code2prompt",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    url="https://github.com/mufeedvh/code2prompt",
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7",
)