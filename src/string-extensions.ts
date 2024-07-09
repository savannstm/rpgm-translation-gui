String.prototype.replaceAllMultiple = function (replacementObj) {
    return this.replaceAll(Object.keys(replacementObj).join("|"), (match) => replacementObj[match]);
};

String.prototype.count = function (char) {
    let occurrences = 0;

    for (let i = 0; i < this.length; i++) {
        if (char === this[i]) {
            occurrences++;
        }
    }

    return occurrences;
};
