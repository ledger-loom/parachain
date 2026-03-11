/**
 * API Example: Role-Based Encrypted Product Management
 *
 * This example shows how to:
 * 1. Encrypt product data on the client/backend
 * 2. Store encrypted data on the parachain
 * 3. Retrieve and decrypt based on user roles
 * 4. Integrate with your existing role-permissions pallet
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { ProductEncryption, EncryptionHelper } = require('./encryption-helpers');

/**
 * Supply Chain API with Encryption
 */
class SupplyChainEncryptedAPI {
    constructor(api, keyring) {
        this.api = api;
        this.keyring = keyring;
        // In production, store keys securely (AWS KMS, HashiCorp Vault, etc.)
        this.companyKeys = new Map(); // company_id => encryption_key
    }

    /**
     * Register a company encryption key
     */
    async registerCompanyKey(companyId, adminAccount) {
        // Generate encryption key for company
        const encryptionKey = EncryptionHelper.generateKey();

        // Store locally (in production: use secure key management)
        this.companyKeys.set(companyId, encryptionKey);

        // Generate key ID and hash
        const keyId = EncryptionHelper.generateKeyId(encryptionKey);
        const keyHash = EncryptionHelper.hash(encryptionKey);

        // Register key metadata on-chain (NOT the actual key!)
        const tx = this.api.tx.encryptedData.registerCompanyKey(
            companyId,
            keyId,
            Array.from(keyHash),
            'AES256' // EncryptionAlgorithm enum
        );

        await this.signAndSend(tx, adminAccount);

        console.log(`✅ Company ${companyId} encryption key registered`);
        console.log(`   Key ID: ${keyId}`);

        return { keyId, encryptionKey };
    }

    /**
     * Create encrypted product
     */
    async createEncryptedProduct(productData, userAccount) {
        const { company_id, name, attributes, category, visibility, authorized_roles } = productData;

        // Get company encryption key
        const companyKey = this.companyKeys.get(company_id);
        if (!companyKey) {
            throw new Error(`No encryption key found for company ${company_id}`);
        }

        // Encrypt sensitive data
        const encrypted = ProductEncryption.encryptProduct(
            { name, attributes, category, company_id },
            companyKey
        );

        // Submit to blockchain
        const tx = this.api.tx.productManagement.createEncryptedProduct(
            company_id,
            encrypted.encrypted_name,
            encrypted.encrypted_attributes,
            category,
            encrypted.data_hash,
            encrypted.encryption_key_id,
            visibility || 'Company', // VisibilityLevel enum
            authorized_roles || []
        );

        const result = await this.signAndSend(tx, userAccount);

        // Extract product ID from events
        const productId = this.extractProductId(result.events);

        console.log(`✅ Encrypted product created: ID ${productId}`);
        console.log(`   Name: ${name} (stored encrypted)`);
        console.log(`   Category: ${category} (public)`);
        console.log(`   Visibility: ${visibility}`);

        return { productId, dataHash: encrypted.data_hash };
    }

    /**
     * Get product with role-based decryption
     */
    async getProduct(productId, userId, userRole, userCompanyId) {
        // 1. Check access on-chain first
        const userAccount = this.keyring.getPair(userId);

        try {
            const tx = this.api.tx.productManagement.accessProduct(
                productId,
                userRole,
                userCompanyId
            );
            await this.signAndSend(tx, userAccount);
        } catch (error) {
            throw new Error('Access denied: ' + error.message);
        }

        // 2. Fetch encrypted product from chain
        const product = await this.api.query.productManagement.products(productId);

        if (product.isNone) {
            throw new Error('Product not found');
        }

        const encryptedProduct = product.unwrap().toJSON();

        // 3. If not encrypted, return as is
        if (!encryptedProduct.is_encrypted) {
            return encryptedProduct;
        }

        // 4. Decrypt based on user role
        const companyKey = this.companyKeys.get(userCompanyId);
        if (!companyKey) {
            // User doesn't have decryption key
            return {
                ...encryptedProduct,
                name: '[Encrypted]',
                attributes: '[Encrypted]'
            };
        }

        // Role-based decryption
        const decrypted = ProductEncryption.decryptForRole(
            encryptedProduct,
            companyKey,
            this.getRoleName(userRole)
        );

        console.log(`✅ Product ${productId} accessed by role: ${this.getRoleName(userRole)}`);

        return decrypted;
    }

    /**
     * Get all products for a company (with role filtering)
     */
    async getCompanyProducts(companyId, userId, userRole) {
        const productIds = await this.api.query.productManagement.companyProducts.keys(companyId);

        const products = [];
        for (const key of productIds) {
            const productId = key.args[1].toNumber();

            try {
                const product = await this.getProduct(productId, userId, userRole, companyId);
                products.push(product);
            } catch (error) {
                console.log(`Skipping product ${productId}: ${error.message}`);
            }
        }

        return products;
    }

    /**
     * Update product visibility
     */
    async updateVisibility(productId, newVisibility, authorizedRoles, userAccount) {
        const tx = this.api.tx.productManagement.updateVisibility(
            productId,
            newVisibility,
            authorizedRoles
        );

        await this.signAndSend(tx, userAccount);

        console.log(`✅ Product ${productId} visibility updated to: ${newVisibility}`);
    }

    // Helper methods
    getRoleName(roleId) {
        const roles = { 1: 'admin', 2: 'manager', 3: 'sales', 4: 'viewer' };
        return roles[roleId] || 'unknown';
    }

    async signAndSend(tx, account) {
        return new Promise((resolve, reject) => {
            tx.signAndSend(account, ({ status, events }) => {
                if (status.isInBlock) {
                    resolve({ events });
                } else if (status.isFinalized) {
                    console.log(`Transaction finalized in block: ${status.asFinalized}`);
                }
            }).catch(reject);
        });
    }

    extractProductId(events) {
        for (const { event } of events) {
            if (event.section === 'productManagement' &&
                event.method === 'ProductCreatedEncrypted') {
                return event.data[0].toNumber();
            }
        }
        return null;
    }
}

/**
 * Example Usage
 */
async function main() {
    console.log('=== Supply Chain Encrypted API Example ===\n');

    // 1. Connect to parachain
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: 'sr25519' });

    // 2. Initialize API
    const scAPI = new SupplyChainEncryptedAPI(api, keyring);

    // 3. Setup accounts
    const adminAccount = keyring.addFromUri('//Alice');
    const managerAccount = keyring.addFromUri('//Bob');
    const salesAccount = keyring.addFromUri('//Charlie');

    console.log('1. Setting up accounts...');
    console.log(`   Admin: ${adminAccount.address}`);
    console.log(`   Manager: ${managerAccount.address}`);
    console.log(`   Sales: ${salesAccount.address}\n`);

    // 4. Register company encryption key
    console.log('2. Registering company encryption key...');
    const companyId = 1;
    await scAPI.registerCompanyKey(companyId, adminAccount);
    console.log('');

    // 5. Create encrypted product
    console.log('3. Creating encrypted product...');
    const productData = {
        company_id: companyId,
        name: 'Organic Ethiopian Coffee',
        attributes: {
            supplier: 'Farm ABC, Yirgacheffe',
            origin: 'Ethiopia',
            price: '$45/kg',
            cost: '$30/kg',
            quality_score: 95,
            batch: '2024-ETH-001',
            certifications: ['USDA Organic', 'Fair Trade']
        },
        category: 'Food & Beverage',
        visibility: 'Company', // Only company members can access
        authorized_roles: [1, 2, 3] // Admin, Manager, Sales
    };

    const { productId } = await scAPI.createEncryptedProduct(productData, adminAccount);
    console.log('');

    // 6. Access product as different roles
    console.log('4. Accessing product with different roles:\n');

    // Manager (full access)
    console.log('   As Manager:');
    const productAsManager = await scAPI.getProduct(
        productId,
        managerAccount.address,
        2, // manager role
        companyId
    );
    console.log(`   - Name: ${productAsManager.name}`);
    console.log(`   - Supplier: ${productAsManager.attributes?.supplier}`);
    console.log(`   - Price: ${productAsManager.attributes?.price}`);
    console.log(`   - Quality: ${productAsManager.attributes?.quality_score}\n`);

    // Sales (limited access)
    console.log('   As Sales:');
    const productAsSales = await scAPI.getProduct(
        productId,
        salesAccount.address,
        3, // sales role
        companyId
    );
    console.log(`   - Name: ${productAsSales.name}`);
    console.log(`   - Supplier: ${productAsSales.attributes?.supplier || '[Hidden]'}`);
    console.log(`   - Price: ${productAsSales.attributes?.price || '[Hidden]'}`);
    console.log(`   - Quality: ${productAsSales.attributes?.quality_score}\n`);

    // 7. Update visibility
    console.log('5. Updating product visibility...');
    await scAPI.updateVisibility(
        productId,
        'Management', // Only managers and admins
        [1, 2], // Admin, Manager only
        adminAccount
    );
    console.log('');

    // 8. Try to access as sales (should fail now)
    console.log('6. Attempting to access as Sales (should be denied)...');
    try {
        await scAPI.getProduct(productId, salesAccount.address, 3, companyId);
    } catch (error) {
        console.log(`   ❌ ${error.message}\n`);
    }

    console.log('✅ Example completed successfully!');

    await api.disconnect();
}

// Run example
if (require.main === module) {
    main().catch(console.error);
}

module.exports = { SupplyChainEncryptedAPI };
