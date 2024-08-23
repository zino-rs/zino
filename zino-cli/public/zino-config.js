// fetch the current directory and Cargo.toml content
const currentDirInput = document.getElementById('currentDir');

currentDirInput.addEventListener('blur', updateCurrentDir);

currentDirInput.addEventListener('keyup', async function (event) {
    if (event.key === 'Enter') {
        event.preventDefault(); // 防止默认的回车行为（如提交表单）
        await updateCurrentDir();
    }
});

async function updateCurrentDir() {
    const currentDir = currentDirInput.value;

    try {
        const response = await fetch(`/update_current_dir/${encodeURIComponent(currentDir)}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain',
            },
        });
        if (!response.ok) {
            throw new Error(await response.text());
        }
    } catch (err) {
        console.error('Failed to update directory:', err);
        await fetchCurrentDir();
    }

    await fetchCargoToml()
    await fetchFeatures()
}

// ask the server to change the current directory
async function fetchCurrentDir() {
    try {
        const response = await fetch('/current_dir');
        if (!response.ok) {
            throw new Error((await response.json()).data);
        }
        document.getElementById('currentDir').value = (await response.json()).data;
    } catch (error) {
        console.error('Failed to fetch current directory:', error);
    }
}

// get the content of current_dir/Cargo.toml
async function fetchCargoToml() {
    try {
        const response = await fetch('/get_current_cargo_toml');
        if (!response.ok) {
            throw new Error((await response.json()).data);
        }
        const content = (await response.json()).data;
        document.getElementById('currentCargoTomlTextArea').value = content;

        const packageNameLine = content.split('\n').find(line => line.startsWith('name ='));
        const projectName = packageNameLine ? packageNameLine.split('=')[1].trim().replace(/"/g, '') : 'Not Found';
        document.getElementById("project_name").textContent = `current project: ${projectName}`;
        updateLineNumbers(document.getElementById('currentCargoTomlTextArea'));
    } catch (error) {
        console.error('Failed to fetch Cargo.toml:', error);
        document.getElementById('currentCargoTomlDescription').value = 'Failed to fetch Cargo.toml, make sure you entered a valid project directory';
    }
}


// init the options of each group
async function fetchFeatures() {
    try {
        const response = fetch('/get_current_features');
        let features = await (await response).json();
        document.querySelectorAll('.checked').forEach(option => {
            option.classList.replace('checked', 'unchecked')
        })
        for (let feature of features.data.zino_feature) {
            Array.from(document.querySelectorAll('#zino-config-form [data-feature]')).filter(option => option.getAttribute('data-feature') === feature).forEach(option => {
                option.click()
            })
        }
        for (let feature of features.data.core_feature) {
            Array.from(document.querySelectorAll('#core-config-form [data-feature]')).filter(option => option.getAttribute('data-feature') === feature).forEach(option => {
                option.click()
            })
        }
    } catch (error) {
        console.error('Failed to init options:', error);
    }
}

function checkedOptions() {
    let option_groups = {};
    document.querySelectorAll('.closet').forEach(closet => {
        const groupName = closet.querySelector('.option-title').textContent;
        option_groups[groupName] = [];
        closet.querySelectorAll('.checked').forEach(option => {
            if (!option.classList.contains('all-options')) {
                option_groups[groupName].push(option.getAttribute('data-feature'));
            } else {
                let all_flag = option.getAttribute('data-feature')
                if (all_flag != null) {
                    option_groups[groupName] = [option.getAttribute('data-feature')]
                }
            }
        });
    });
    let option = {
        zino_feature: option_groups['Framework']
            .concat(option_groups['zino-features'])
            .sort(),
        core_feature: option_groups['core-features']
            .concat(option_groups['Database'])
            .concat(option_groups['Accessor'])
            .concat(option_groups['Connector'])
            .concat(option_groups['locale'])
            .concat(option_groups['validator'])
            .concat(option_groups['view'])
            .sort()
    }
    return option;
}

async function generateCargoToml() {
    const res = await fetch('/generate_cargo_toml', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(checkedOptions())
    });


    document.querySelector('#aimCargoTomlTextArea').value = (await res.json()).data;
}


function change_option_state() {
    this.classList.toggle('unchecked');
    this.classList.toggle('checked');
    let self_checked = this.classList.contains('checked');

    let group = this.parentElement;

    if (group.classList.contains('exclusive')) {
        [...group.querySelectorAll('.checked')].filter(o => o !== this).forEach(option => {
            option.classList.toggle('checked')
            option.classList.toggle('unchecked')
        })
    }

    if (this.classList.contains('all-options')) {
        if (self_checked) {
            group.querySelectorAll('.unchecked').forEach(option => {
                option.classList.replace('unchecked', 'checked')
            })
        } else {
            group.querySelectorAll('.checked').forEach(option => {
                option.classList.replace('checked', 'unchecked')
            })
        }
    } else {
        if (self_checked) {
            if (group.querySelectorAll('.unchecked').length === 1) {
                group.querySelectorAll('.all-options').forEach(option => {
                    option.classList.replace('unchecked', 'checked')
                })
            }
        } else {
            group.querySelectorAll('.all-options').forEach(option => {
                option.classList.replace('checked', 'unchecked')
            })
        }
    }

    const ormOption = document.querySelector('#zino-config-form [data-feature="orm"]');
    const ormForm = document.getElementById('orm-form');
    if (ormOption && ormOption.classList.contains('checked')) {
        ormForm.classList.remove('hidden');
    } else {
        ormForm.classList.add('hidden');
        ormForm.querySelectorAll('.option-group div').forEach(option => {
            option.classList.replace('checked', 'unchecked');
        });
    }

    generateCargoToml();
}

document.querySelectorAll('.unchecked').forEach(option => {
    option.addEventListener('click', change_option_state);
});


function updateLineNumbers(textArea) {
    const lineCount = textArea.value.split('\n').length;
    let lineNumberHtml = '';
    for (let i = 1; i <= lineCount; i++) {
        lineNumberHtml += `${i}\n`;
    }

    const lineNumberElement = textArea.previousElementSibling;
    lineNumberElement.textContent = lineNumberHtml;
}

document.querySelectorAll('.cargoTomlTextArea').forEach(textArea => {
    textArea.addEventListener('input', function () {
        updateLineNumbers(this);
    });
    textArea.addEventListener('scroll', function () {
        this.previousElementSibling.style.marginTop = `-${this.scrollTop}px`;
    });
});

// save the generated Cargo.toml
document.getElementById('save-config').addEventListener('click', async () => {
    const aimCargoToml = document.getElementById('aimCargoTomlTextArea').value;
    try {
        const response = await fetch('/save_cargo_toml', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(aimCargoToml),
        });
        if (!response.ok) {
            throw new Error(await response.text());
        }
        await fetchCargoToml();
    } catch (error) {
        console.error('Failed to save Cargo.toml:', error);
    }
});

window.onload = async () => {
    await fetchCurrentDir();
    await fetchCargoToml();
    await fetchFeatures();
};