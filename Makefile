SCRIPT_FILES := $(wildcard js/*.js)
BASE_SCRIPT_FILES := $(basename $(SCRIPT_FILES))
SCRIPT_TEMPLATES := $(addsuffix .template, $(BASE_SCRIPT_FILES))

.DEFAULT_GOAL := index.html

header.template: head_begin.template style.css head_end.template
	cat head_begin.template style.css head_end.template > header.template

$(SCRIPT_TEMPLATES): $(SCRIPT_FILES) script_start.template script_end.template
	cat script_start.template $(addsuffix .js, $(basename $(@))) script_end.template > $@

scripts.template: $(SCRIPT_TEMPLATES)
	cat $(SCRIPT_TEMPLATES) > scripts.template

body.template: body.html scripts.template body_begin.template body_end.template
	cat body_begin.template body.html scripts.template body_end.template > body.template

index.html: header.template body.template tail.template
	cat header.template  body.template tail.template > index.html
