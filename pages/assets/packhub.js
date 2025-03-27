function process_copy_button_click(id, klass) {
    try {
        const command = document.querySelector(`.command.${klass}`);
        navigator.clipboard.writeText(command.textContent).then(() =>
          document.getElementById(id).style.opacity = '1');

        setTimeout(() => document.getElementById(id).style.opacity = '0', 3000);
    } catch (e) {
        console.log('Hit a snag when copying to clipboard: ', e);
    }
}

function handle_copy_button_click(e) {
    switch (e.id) {
        case 'copy-button-ubuntu':
            process_copy_button_click('copy-status-message-ubuntu', 'ubuntu');
            break;
        case 'copy-button-debian':
            process_copy_button_click('copy-status-message-debian', 'debian');
            break;
        case 'copy-button-fedora':
            process_copy_button_click('copy-status-message-fedora', 'fedora');
            break;
    }
}

function set_up_copy_button_clicks() {
    var buttons = document.querySelectorAll(".copy-button");
    buttons.forEach(function (element) {
        element.addEventListener('click', function() {
            handle_copy_button_click(element);
        });
    })
}

set_up_copy_button_clicks();

function set_ubuntu(owner, repo) {
    const ubuntu = document.querySelector(".command.ubuntu");

    console.log("Setting Ubuntu install command for:", ubuntu);

    ubuntu.textContent = `wget -qO- http://packhub.dev/sh/ubuntu/github/${owner}/${repo} | sh`
}

function set_debian(owner, repo) {
    const ubuntu = document.querySelector(".command.debian");

    console.log("Setting Ubuntu install command for:", ubuntu);

    ubuntu.textContent = `wget -qO- http://packhub.dev/sh/debian/github/${owner}/${repo} | sh`
}

document.addEventListener("DOMContentLoaded", () => {
    const inputElement = document.querySelector(".github-link");

    function extractGithubInfo(value) {
        const githubRegex = /https?:\/\/github\.com\/([^\/]+)\/([^\/]+)/;
        const match = value.match(githubRegex);
        
        if (match) {
            set_ubuntu(match[1], match[2]);
            set_debian(match[1], match[2]);
        } else {
            console.log("Invalid or missing GitHub URL");
        }
    }

    if (inputElement) {
        // Extract initial value (if present)
        extractGithubInfo(inputElement.value || inputElement.placeholder);
        
        // Listen for changes in the input field
        inputElement.addEventListener("input", (event) => {
            extractGithubInfo(inputElement.value || inputElement.placeholder);
        });
    }
});

