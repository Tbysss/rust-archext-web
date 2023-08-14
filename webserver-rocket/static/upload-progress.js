window.onload = function() {
    document.getElementsByTagName('form')[0].addEventListener('submit', async e => {
            e.preventDefault()

            const uploadFormData = new FormData(e.target)
            const filenames = uploadFormData.getAll('submission.file').map(v => v.name).join(', ')
            const uploadRequest = new XMLHttpRequest()
            uploadRequest.open(e.target.method, e.target.action)
            uploadRequest.timeout = 3600000
            
            uploadRequest.upload.onprogress = e => {
                let message = e.loaded === e.total ? 'Saving…' : `${Math.floor(100*e.loaded/e.total)}% [${e.loaded >> 10} / ${e.total >> 10}KiB]`
                document.getElementById("upload-submit").value = message
                document.getElementById("upload-submit").disabled = true
            }

            uploadRequest.onreadystatechange = () => {
                if(uploadRequest.readyState == 4) {
                    document.write(uploadRequest.response)
                }
            }
        
            uploadRequest.send(uploadFormData)
    })
}