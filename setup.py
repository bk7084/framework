import setuptools

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setuptools.setup(
    name='bk7084',
    version='0.2.3',
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
    package_data={
        'bk7084': ['assets/models/*', 'assets/shaders/*', 'assets/textures/*']
    },
    python_requires='>=3.9,<3.11',
    install_requires=[
        'numpy',
        'pyopengl',
        'glfw',
        'pypng',
        'imgui[glfw]',
        'Pillow',
        'numba'
    ],
)
