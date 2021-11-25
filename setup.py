import setuptools

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setuptools.setup(
    name='bk7084',
    version='0.1.3',
    author='Yang Chen, Ruben Wiersma, Ricardo Marroquim',
    author_email="matthiasychen@gmail.com, rubenwiersma@gmail.com, R.Marroquim@tudelft.nl",
    description='Python framework for BK7084 Computational Simulations',
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/bk7084/framework",
    project_urls={
        "Bug Tracker": "https://github.com/bk7084/framework/issues",
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
    ],
    packages=setuptools.find_packages(where=".", include=["bk7084", "bk7084.*"]),
    python_requires='>=3.9',
    install_requires=[
        'numpy',
        'pyopengl',
        'glfw',
        'pypng',
        'imgui[glfw]',
        'scipy',
        'trimesh'
    ],
)
