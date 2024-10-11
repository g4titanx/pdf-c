# Experimental PDF Compressor Function

**IMPORTANT**: This PDF compression function is experimental and intended for demonstration purposes only. It may not be suitable for production use without further testing and refinement.

Follow these steps to compress a PDF using the Fleek function and extract the result:

- Call the Fleek function:

   Replace `<your-hash>` with the hash provided when you deployed your function.

   ```bash
   curl fleek-test.network/services/3 --data '{"hash": "<your-hash>", "decrypt": true, "input": "'"$(cat test_base64.txt)"'"}' --output response.bin
   ```

- Extract the PDF content:

   The response contains metadata followed by the compressed PDF.

   ```bash
   cat response.bin | sed '1d' > compressed.pdf
   ```
