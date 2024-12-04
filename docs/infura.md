# Infura Guide

## Quick Links

- [How to get your Infura API key](#how-to-get-your-infura-api-key)
- [How to Switch from Alchemy Key to Infura Key Using CLI](#how-to-switch-from-alchemy-key-to-infura-key-using-cli)

## How to get your Infura API key

1. Go to <a href="https://www.infura.io/" target="_blank">Infura</a> and click on `Get Started` button.
<div align="center"><img src="../assets/infura/a1.png" width="800"/></div>

2. Click on `Sign up today` for an account or log in if you already have one.

<div align="center"><img src="../assets/infura/a2.png" width="800"/></div>

3. If you are a new user, you will be asked to fill in your details. Fill all the boxes and click on `Create a free account` button.

<div align="center"><img src="../assets/infura/a3.png" width="800"/></div>

4. Verify that you are a human being by clicking on the checkbox.

<div align="center"><img src="../assets/infura/a4.png" width="800"/></div>

5. Select option and answer as shown in the image below.

<div align="center"><img src="../assets/infura/a5.png" width="800"/></div>

6. Select category as shown in the image below.

<div align="center"><img src="../assets/infura/a6.png" width="800"/></div>

7. Select the plan. In this case, we will select the free plan. Click on `Start building`.

<div align="center"><img src="../assets/infura/a7.png" width="800"/></div>

8. You will be redirected to the dashboard. In the center of the page, you will see the dropdown box to select network.

<div align="center"><img src="../assets/infura/a8.png" width="800"/></div>

9. Choose `base`, then click the `next` button.

<div align="center"><img src="../assets/infura/a9.png" width="800"/></div>

10. In the center of the page, you will see your API key. Copy it and paste it to the CLI when asked.

<div align="center"><img src="../assets/infura/a10.png" width="800"/></div>

That's it! You have successfully obtained your Infura API key.


# How to Switch from Alchemy Key to Infura Key Using CLI

1. **Select Network**:
   When you open the CLI, you'll see a screen like this, select `base`:

  <div align="center">
    <img src="../assets/infura/b1.png" width="600" alt="Mining CLI"></div>

2. **Modify Option**:
   After selecting `base`, you'll see a screen like this:

  <div align="center">
    <img src="../assets/infura/b2.png" width="600" alt="Mining CLI"></div>

  If you choose to encrypt your password during setup, you'll be prompted to enter it. This step is optional and won't appear if you didn't set a password.

3. **Modify API Key**: Type `y` to change Api Key.

<div align="center"><img src="../assets/infura/b3.png" width="600" alt="Mining CLI"></div>


4. **Select Infura**: Select `Infura`.

<div align="center"><img src="../assets/infura/b4.png" width="600" alt="Mining CLI"></div>


5. **Select Infura**: Enter your new API key which you obtained from [How to get your Infura API key](#how-to-get-your-infura-api-key) guide above.

<div align="center"><img src="../assets/infura/b5.png" width="600" alt="Mining CLI"></div>

Note: Copy just the API key.

<div align="center"><img src="../assets/infura/b6.png" width="600" alt="Mining CLI"></div>

Note that your API key will not be displayed. Press `Enter` after pasting it.

<div align="center"><img src="../assets/infura/b7.png" width="600" alt="Mining CLI"></div>

To modify other options such as the gas price or withdrawal address, type `n`.

You can also choose to encrypt your private key by setting a password. Encryption is highly recommended for security. Press `y` or `Enter` to encrypt the private key, or `n` to save it as plain text.

6. **Continue Mining Activity**: The API key has been successfully switched from Alchemy to Infura. You can now proceed with the mining options.

<div align="center"><img src="../assets/infura/b8.png" width="600" alt="Mining CLI"></div>