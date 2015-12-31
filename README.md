Helianto
========

A minimalist website generator written in [Rust][rust].


Installation
------------

You can install Helianto with [Cargo][cargo] :

    cargo install helianto --git="https://github.com/m-r-r/helianto.git"


Basic usage
-----------

### Creating a new project

One you have installed Helianto, you can create a new project with the `--init`
option:

    helianto --init new-website

Helianto will create a directory and populate it with the default template and 
CSS files.  
An exemple page will also be created.

You can then build your site with the following command:

    cd new-website
    helianto


### Changing the layout

You can change the layout of your website by modifying the `page.html.hbs` file
in the `_layouts` directory. If this file is missing, Helianto will use the
builtin one instead.

All the templates are using the [Handlebar][hbs] syntax.


### Changing the assets

By default, Helianto creates a `css` directory containing the stylesheets used
by the website.

Helianto copies all the files wich are not documents to the output directory.
You can thus edit or remove the existing stylesheets and add new static files.

### Adding content

You can create new pages by adding Markdown files in your website's directory.  
The directory structure created by `helianto --init` already include an example page:

```markdown
# Welcome

Created:  2015-12-30T16:47:45+01:00  
Keywords: helianto, test  

This is an example
```

The metadata block is optional, only the title of the document is required.

For now, Helianto only supports the following metadata :


| Name     | Format                             | Comment                               |
|----------|------------------------------------|---------------------------------------|
| Created  | An RFC 3339 date                   | Used to sort the entries in the index |
| Keywords | A coma separated list of  keywords | Used in the HTML metadata             |
| Language | An ISO 639-1 language code         | Used in the HTML metadata             |



[rust]:  http://rust-lang.org                                       "The Rust programming language"
[cargo]: http://doc.crates.io                                       "Cargo, Rust’s Package Manager"
[hbs]:   https://github.com/sunng87/handlebars-rust#handlebars-rust "Rust templating with Handlebars"
