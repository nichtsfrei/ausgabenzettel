SCRIPT_FILES := $(wildcard js/*.js)
BASE_SCRIPT_FILES := $(basename $(SCRIPT_FILES))
SCRIPT_TEMPLATES := $(addsuffix .template, $(BASE_SCRIPT_FILES))

.DEFAULT_GOAL := index.html

header.template: head_begin.template style.css head_end.template body_begin.template scripts.template body.html
	cat head_begin.template style.css head_end.template body_begin.template scripts.template body.html > header.template

$(SCRIPT_TEMPLATES): $(SCRIPT_FILES) script_start.template script_end.template
	cat script_start.template $(addsuffix .js, $(basename $(@))) script_end.template > $@

scripts.template: $(SCRIPT_TEMPLATES)
	cat $(SCRIPT_TEMPLATES) > scripts.template

index.html: header.template content.html tail.template
	cat header.template content.html tail.template > index.html

install: header.template content.html tail.template
	cp header.template ../backend/initial/head.template
	cp content.html ../backend/initial/content.template
	cp tail.template ../backend/initial/tail.template
