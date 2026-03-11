/**
 * Encryption Helper Utilities for Supply Chain
 *
 * Client-side encryption/decryption for sensitive data before sending to blockchain
 * Uses AES-256-CBC encryption
 */

const crypto = require('crypto');

class EncryptionHelper {
    /**
     * Encrypt data with AES-256-CBC
     * @param {string|object} data - Data to encrypt
     * @param {Buffer} key - 32-byte encryption key
     * @returns {object} - {ciphertext, iv, hash}
     */
    static encrypt(data, key) {
        // Convert data to string if object
        const plaintext = typeof data === 'object' ? JSON.stringify(data) : String(data);

        // Generate random IV (initialization vector)
        const iv = crypto.randomBytes(16);

        // Create cipher
        const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);

        // Encrypt
        let encrypted = cipher.update(plaintext, 'utf8', 'hex');
        encrypted += cipher.final('hex');

        // Calculate hash of plaintext for integrity verification
        const hash = crypto.createHash('sha256').update(plaintext).digest();

        return {
            ciphertext: Buffer.from(encrypted, 'hex'),
            iv: iv,
            hash: hash,
            algorithm: 'aes-256-cbc'
        };
    }

    /**
     * Decrypt data with AES-256-CBC
     * @param {Buffer} ciphertext - Encrypted data
     * @param {Buffer} key - 32-byte encryption key
     * @param {Buffer} iv - 16-byte initialization vector
     * @returns {string|object} - Decrypted data
     */
    static decrypt(ciphertext, key, iv) {
        // Create decipher
        const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);

        // Decrypt
        let decrypted = decipher.update(ciphertext, 'hex', 'utf8');
        decrypted += decipher.final('utf8');

        // Try to parse as JSON, otherwise return as string
        try {
            return JSON.parse(decrypted);
        } catch {
            return decrypted;
        }
    }

    /**
     * Generate a new encryption key
     * @returns {Buffer} - 32-byte key
     */
    static generateKey() {
        return crypto.randomBytes(32);
    }

    /**
     * Derive key from password using PBKDF2
     * @param {string} password - Password to derive key from
     * @param {Buffer} salt - Salt for key derivation
     * @returns {Buffer} - Derived 32-byte key
     */
    static deriveKey(password, salt) {
        return crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
    }

    /**
     * Generate key ID (hash of public key)
     * @param {Buffer} key - Encryption key
     * @returns {string} - Hex string key ID
     */
    static generateKeyId(key) {
        return crypto.createHash('sha256').update(key).digest('hex');
    }

    /**
     * Hash data with SHA-256
     * @param {string|Buffer} data - Data to hash
     * @returns {Buffer} - 32-byte hash
     */
    static hash(data) {
        return crypto.createHash('sha256').update(data).digest();
    }
}

/**
 * Product Encryption Wrapper
 * Handles encryption of product data for blockchain storage
 */
class ProductEncryption {
    /**
     * Encrypt product data
     * @param {object} productData - Product information
     * @param {Buffer} companyKey - Company encryption key
     * @returns {object} - Encrypted product ready for blockchain
     */
    static encryptProduct(productData, companyKey) {
        const { name, attributes, ...publicData } = productData;

        // Encrypt sensitive fields
        const encryptedName = EncryptionHelper.encrypt(name, companyKey);
        const encryptedAttrs = EncryptionHelper.encrypt(attributes, companyKey);

        // Generate overall data hash
        const allData = JSON.stringify({ name, attributes });
        const dataHash = EncryptionHelper.hash(allData);

        // Generate key ID
        const keyId = EncryptionHelper.generateKeyId(companyKey);

        return {
            ...publicData,
            encrypted_name: Array.from(encryptedName.ciphertext),
            encrypted_attributes: Array.from(encryptedAttrs.ciphertext),
            data_hash: Array.from(dataHash),
            encryption_key_id: keyId,
            iv_name: Array.from(encryptedName.iv),
            iv_attrs: Array.from(encryptedAttrs.iv),
        };
    }

    /**
     * Decrypt product data
     * @param {object} encryptedProduct - Encrypted product from blockchain
     * @param {Buffer} companyKey - Company encryption key
     * @returns {object} - Decrypted product
     */
    static decryptProduct(encryptedProduct, companyKey) {
        const {
            encrypted_name,
            encrypted_attributes,
            iv_name,
            iv_attrs,
            ...publicData
        } = encryptedProduct;

        // Decrypt fields
        const name = EncryptionHelper.decrypt(
            Buffer.from(encrypted_name),
            companyKey,
            Buffer.from(iv_name)
        );

        const attributes = EncryptionHelper.decrypt(
            Buffer.from(encrypted_attributes),
            companyKey,
            Buffer.from(iv_attrs)
        );

        return {
            ...publicData,
            name,
            attributes
        };
    }

    /**
     * Decrypt product based on user role
     * @param {object} encryptedProduct - Encrypted product from blockchain
     * @param {Buffer} companyKey - Company encryption key
     * @param {string} userRole - User role (admin, manager, sales, viewer)
     * @returns {object} - Partially decrypted product based on role
     */
    static decryptForRole(encryptedProduct, companyKey, userRole) {
        const fullProduct = this.decryptProduct(encryptedProduct, companyKey);

        // Filter fields based on role
        switch (userRole) {
            case 'admin':
            case 'manager':
                // Full access
                return fullProduct;

            case 'sales':
                // No supplier or pricing info
                const { attributes } = fullProduct;
                return {
                    ...encryptedProduct,
                    name: fullProduct.name,
                    attributes: {
                        ...attributes,
                        supplier: undefined,
                        price: undefined,
                        cost: undefined,
                    }
                };

            case 'viewer':
                // Only basic info
                return {
                    ...encryptedProduct,
                    name: fullProduct.name,
                };

            default:
                // No decryption
                return encryptedProduct;
        }
    }
}

/**
 * Tracking Data Encryption
 * Handles encryption of supply chain tracking data
 */
class TrackingEncryption {
    /**
     * Encrypt tracking event
     * @param {object} trackingData - Tracking event data
     * @param {Buffer} companyKey - Company encryption key
     * @returns {object} - Encrypted tracking data
     */
    static encryptTracking(trackingData, companyKey) {
        const { location, notes, handler, ...publicData } = trackingData;

        // Encrypt sensitive fields
        const sensitiveData = { location, notes, handler };
        const encrypted = EncryptionHelper.encrypt(sensitiveData, companyKey);

        const dataHash = EncryptionHelper.hash(JSON.stringify(sensitiveData));
        const keyId = EncryptionHelper.generateKeyId(companyKey);

        return {
            ...publicData,
            encrypted_data: Array.from(encrypted.ciphertext),
            data_hash: Array.from(dataHash),
            encryption_key_id: keyId,
            iv: Array.from(encrypted.iv),
        };
    }

    /**
     * Decrypt tracking event
     * @param {object} encryptedTracking - Encrypted tracking from blockchain
     * @param {Buffer} companyKey - Company encryption key
     * @returns {object} - Decrypted tracking data
     */
    static decryptTracking(encryptedTracking, companyKey) {
        const { encrypted_data, iv, ...publicData } = encryptedTracking;

        const decrypted = EncryptionHelper.decrypt(
            Buffer.from(encrypted_data),
            companyKey,
            Buffer.from(iv)
        );

        return {
            ...publicData,
            ...decrypted
        };
    }
}

// Export classes
module.exports = {
    EncryptionHelper,
    ProductEncryption,
    TrackingEncryption
};

// Example usage
if (require.main === module) {
    console.log('=== Encryption Helper Examples ===\n');

    // 1. Generate company key
    const companyKey = EncryptionHelper.generateKey();
    console.log('1. Company Key Generated:', companyKey.toString('hex').substring(0, 32) + '...\n');

    // 2. Encrypt product
    const productData = {
        name: 'Organic Ethiopian Coffee',
        attributes: {
            supplier: 'Farm ABC',
            origin: 'Yirgacheffe, Ethiopia',
            price: '$45/kg',
            quality_score: 95,
            batch: '2024-ETH-001'
        },
        category: 'Food & Beverage',
        company_id: 1,
    };

    const encryptedProduct = ProductEncryption.encryptProduct(productData, companyKey);
    console.log('2. Encrypted Product:');
    console.log('   Category (public):', encryptedProduct.category);
    console.log('   Encrypted Name:', encryptedProduct.encrypted_name.slice(0, 20), '... (', encryptedProduct.encrypted_name.length, 'bytes)');
    console.log('   Data Hash:', encryptedProduct.data_hash.slice(0, 10), '...\n');

    // 3. Decrypt for different roles
    console.log('3. Role-Based Decryption:\n');

    const forManager = ProductEncryption.decryptForRole(encryptedProduct, companyKey, 'manager');
    console.log('   Manager sees:');
    console.log('   - Name:', forManager.name);
    console.log('   - Supplier:', forManager.attributes.supplier);
    console.log('   - Price:', forManager.attributes.price);
    console.log('   - Quality:', forManager.attributes.quality_score, '\n');

    const forSales = ProductEncryption.decryptForRole(encryptedProduct, companyKey, 'sales');
    console.log('   Sales sees:');
    console.log('   - Name:', forSales.name);
    console.log('   - Supplier:', forSales.attributes.supplier);
    console.log('   - Price:', forSales.attributes.price);
    console.log('   - Quality:', forSales.attributes.quality_score, '\n');

    const forViewer = ProductEncryption.decryptForRole(encryptedProduct, companyKey, 'viewer');
    console.log('   Viewer sees:');
    console.log('   - Name:', forViewer.name);
    console.log('   - Other fields: hidden\n');

    // 4. Tracking encryption
    const trackingData = {
        event_type: 'Shipped',
        location: 'Warehouse B, Dock 5',
        notes: 'Temperature maintained at 2°C',
        handler: 'John Smith, Employee ID: 12345',
        timestamp: Date.now()
    };

    const encryptedTracking = TrackingEncryption.encryptTracking(trackingData, companyKey);
    console.log('4. Encrypted Tracking Event:');
    console.log('   Event Type (public):', trackingData.event_type);
    console.log('   Encrypted Data:', encryptedTracking.encrypted_data.slice(0, 20), '...\n');

    const decryptedTracking = TrackingEncryption.decryptTracking(encryptedTracking, companyKey);
    console.log('5. Decrypted Tracking:');
    console.log('   Location:', decryptedTracking.location);
    console.log('   Handler:', decryptedTracking.handler);
    console.log('   Notes:', decryptedTracking.notes);
}
