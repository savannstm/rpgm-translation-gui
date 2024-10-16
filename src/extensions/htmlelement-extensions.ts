HTMLElement.prototype.toggleMultiple = function (...classes) {
    for (const className of classes) {
        this.classList.toggle(className);
    }
};

HTMLElement.prototype.secondHighestParent = function (childElement) {
    if (!childElement) {
        return childElement;
    }

    let parent = childElement.parentElement!;
    let previous = childElement;

    while (parent !== this) {
        previous = parent;
        parent = parent.parentElement!;
    }

    return previous;
};

HTMLTextAreaElement.prototype.calculateHeight = function () {
    const lineBreaks = this.value.count("\n") + 1;

    const { lineHeight, paddingTop, paddingBottom, borderTopWidth, borderBottomWidth } = window.getComputedStyle(this);

    const newHeight =
        lineBreaks * Number.parseFloat(lineHeight) +
        Number.parseFloat(paddingTop) +
        Number.parseFloat(paddingBottom) +
        Number.parseFloat(borderTopWidth) +
        Number.parseFloat(borderBottomWidth);

    for (const child of this.parentElement?.children ?? []) {
        (child as HTMLElement).style.height = `${newHeight}px`;
    }
};
